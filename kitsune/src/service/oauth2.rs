use super::url::UrlService;
use crate::{
    error::{Error, Result},
    util::generate_secret,
};
use askama::Template;
use askama_axum::IntoResponse;
use axum::response::Response;
use diesel_async::RunQueryDsl;
use http::StatusCode;
use kitsune_db::{
    model::oauth2::{self, NewApplication, NewAuthorizationCode},
    schema::oauth2_applications,
    PgPool,
};
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
    application: oauth2::Application,
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
    db_conn: PgPool,
    url_service: UrlService,
}

impl Oauth2Service {
    pub async fn create_app(&self, create_app: CreateApp) -> Result<oauth2::Application> {
        let mut db_conn = self.db_conn.get().await?;

        diesel::insert_into(oauth2_applications::table)
            .values(NewApplication {
                id: Uuid::now_v7(),
                secret: generate_secret().as_str(),
                name: create_app.name.as_str(),
                redirect_uri: create_app.redirect_uris.as_str(),
                scopes: "",
                website: None,
            })
            .execute(&mut db_conn)
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
        let authorization_code = NewAuthorizationCode {
            code: generate_secret().as_str(),
            application_id: application.id,
            user_id,
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
