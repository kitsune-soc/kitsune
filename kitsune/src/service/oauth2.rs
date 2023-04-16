use super::url::UrlService;
use crate::{
    error::{Error, Result},
    util::generate_secret,
};
use askama::Template;
use askama_axum::IntoResponse;
use axum::response::Response;
use http::StatusCode;
use kitsune_db::entity::{oauth2_applications, oauth2_authorization_codes};
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel};
use std::str::FromStr;
use time::{Duration, OffsetDateTime};
use typed_builder::TypedBuilder;
use url::Url;
use uuid::Uuid;

pub static TOKEN_VALID_DURATION: Duration = Duration::hours(1);

/// If the Redirect URI is equal to this string, show the token instead of redirecting the user
const SHOW_TOKEN_URI: &str = "urn:ietf:wg:oauth:2.0:oob";

#[derive(Clone, TypedBuilder)]
pub struct AuthorisationCode {
    application: oauth2_applications::Model,
    state: Option<String>,
    user_id: Uuid,
}

#[derive(Clone, TypedBuilder)]
pub struct CreateApp {
    name: String,
    redirect_uris: String,
}

#[derive(Template)]
#[template(path = "oauth/token.html")]
struct ShowTokenPage {
    app_name: String,
    domain: String,
    token: String,
}

#[derive(Clone, TypedBuilder)]
pub struct Oauth2Service {
    db_conn: DatabaseConnection,
    url_service: UrlService,
}

impl Oauth2Service {
    pub async fn create_app(&self, create_app: CreateApp) -> Result<oauth2_applications::Model> {
        oauth2_applications::Model {
            id: Uuid::now_v7(),
            secret: generate_secret(),
            name: create_app.name,
            redirect_uri: create_app.redirect_uris,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
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
            created_at: OffsetDateTime::now_utc(),
            expired_at: OffsetDateTime::now_utc() + TOKEN_VALID_DURATION,
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
