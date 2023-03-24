use super::url::UrlService;
use crate::{
    error::{Error, Result},
    util::generate_secret,
};
use askama::Template;
use askama_axum::IntoResponse;
use axum::response::Response;
use chrono::{Duration, Utc};
use derive_builder::Builder;
use http::StatusCode;
use kitsune_db::entity::{oauth2_applications, oauth2_authorization_codes};
use once_cell::sync::Lazy;
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel};
use std::str::FromStr;
use url::Url;
use uuid::Uuid;

pub static TOKEN_VALID_DURATION: Lazy<Duration> = Lazy::new(|| Duration::hours(1));

/// If the Redirect URI is equal to this string, show the token instead of redirecting the user
const SHOW_TOKEN_URI: &str = "urn:ietf:wg:oauth:2.0:oob";

#[derive(Builder, Clone)]
pub struct AuthorisationCode {
    application: oauth2_applications::Model,
    state: Option<String>,
    user_id: Uuid,
}

impl AuthorisationCode {
    #[must_use]
    pub fn builder() -> AuthorisationCodeBuilder {
        AuthorisationCodeBuilder::default()
    }
}

#[derive(Builder, Clone)]
pub struct CreateApp {
    name: String,
    redirect_uris: String,
}

impl CreateApp {
    #[must_use]
    pub fn builder() -> CreateAppBuilder {
        CreateAppBuilder::default()
    }
}

#[derive(Template)]
#[template(path = "oauth/token.html")]
struct ShowTokenPage {
    app_name: String,
    domain: String,
    token: String,
}

#[derive(Builder, Clone)]
pub struct Oauth2Service {
    db_conn: DatabaseConnection,
    url_service: UrlService,
}

impl Oauth2Service {
    #[must_use]
    pub fn builder() -> Oauth2ServiceBuilder {
        Oauth2ServiceBuilder::default()
    }

    pub async fn create_app(&self, create_app: CreateApp) -> Result<oauth2_applications::Model> {
        oauth2_applications::Model {
            id: Uuid::now_v7(),
            secret: generate_secret(),
            name: create_app.name,
            redirect_uri: create_app.redirect_uris,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }
        .into_active_model()
        .insert(&self.db_conn)
        .await
        .map_err(Error::from)
    }

    pub async fn create_authorisation_code_response(
        &self,
        AuthorisationCode {
            application,
            state,
            user_id,
        }: AuthorisationCode,
    ) -> Result<Response> {
        let authorization_code = oauth2_authorization_codes::Model {
            code: generate_secret(),
            application_id: application.id,
            user_id,
            created_at: Utc::now().into(),
            expired_at: (Utc::now() + *TOKEN_VALID_DURATION).into(),
        }
        .into_active_model()
        .insert(&self.db_conn)
        .await?;

        if application.redirect_uri == SHOW_TOKEN_URI {
            Ok(ShowTokenPage {
                app_name: application.name,
                domain: self.url_service.domain().into(),
                token: authorization_code.code,
            }
            .into_response())
        } else {
            let mut url = Url::from_str(&application.redirect_uri)?;
            url.query_pairs_mut()
                .append_pair("code", &authorization_code.code);

            if let Some(state) = state {
                url.query_pairs_mut().append_pair("state", &state);
            }

            Ok((StatusCode::FOUND, [("Location", url.as_str())]).into_response())
        }
    }
}
