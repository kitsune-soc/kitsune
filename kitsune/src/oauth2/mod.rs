use axum::response::{Html, IntoResponse, Redirect, Response};
use diesel_async::RunQueryDsl;
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    model::oauth2,
    schema::{oauth2_applications, oauth2_authorization_codes},
    with_connection, PgPool,
};
use kitsune_derive::kitsune_service;
use kitsune_error::{Error, Result};
use kitsune_url::UrlService;
use kitsune_util::generate_secret;
use serde::Serialize;
use speedy_uuid::Uuid;
use std::str::{self, FromStr};
use strum::{AsRefStr, EnumIter, EnumMessage, EnumString};
use time::Duration;
use typed_builder::TypedBuilder;
use url::Url;

mod auth_code;
mod client_extractor;
mod code_grant;
mod refresh;
mod registrar;

pub use self::{
    auth_code::Issuer as AuthIssuer, client_extractor::Extractor as ClientExtractor,
    code_grant::Issuer as CodeGrantIssuer,
};

/// If the Redirect URI is equal to this string, show the token instead of redirecting the user
const SHOW_TOKEN_URI: &str = "urn:ietf:wg:oauth:2.0:oob";

static AUTH_CODE_VALID_DURATION: Duration = Duration::minutes(10);
static TOKEN_VALID_DURATION: Duration = Duration::hours(1);

#[derive(AsRefStr, Clone, Copy, Debug, EnumIter, EnumMessage, EnumString, Serialize)]
#[strum(serialize_all = "lowercase")]
pub enum OAuthScope {
    #[strum(message = "Read admin-related data", serialize = "admin:read")]
    AdminRead,
    #[strum(message = "Write admin-related data", serialize = "admin:write")]
    AdminWrite,
    #[strum(message = "Read on your behalf")]
    Read,
    #[strum(message = "Write on your behalf")]
    Write,
}

#[derive(Clone, TypedBuilder)]
pub struct AuthorisationCode {
    application: oauth2::Application,
    scopes: Scope,
    state: Option<String>,
    user_id: Uuid,
}

#[derive(Clone, TypedBuilder)]
pub struct CreateApp {
    name: String,
    redirect_uris: String,
}

#[kitsune_service]
pub struct OAuth2Service {
    db_pool: PgPool,
    url_service: UrlService,
}

impl OAuth2Service {
    pub async fn create_app(&self, create_app: CreateApp) -> Result<oauth2::Application> {
        let secret = generate_secret();
        with_connection!(self.db_pool, |db_conn| {
            diesel::insert_into(oauth2_applications::table)
                .values(oauth2::NewApplication {
                    id: Uuid::now_v7(),
                    secret: secret.as_str(),
                    name: create_app.name.as_str(),
                    redirect_uri: create_app.redirect_uris.as_str(),
                    scopes: "",
                    website: None,
                })
                .get_result(db_conn)
                .await
        })
        .map_err(Error::from)
    }

    pub async fn create_authorisation_code_response(
        &self,
        AuthorisationCode {
            application,
            scopes,
            state,
            user_id,
        }: AuthorisationCode,
    ) -> Result<Response> {
        let secret = generate_secret();
        let scopes = scopes.to_string();

        let authorization_code: oauth2::AuthorizationCode =
            with_connection!(self.db_pool, |db_conn| {
                diesel::insert_into(oauth2_authorization_codes::table)
                    .values(oauth2::NewAuthorizationCode {
                        code: secret.as_str(),
                        application_id: application.id,
                        user_id,
                        scopes: scopes.as_str(),
                        expires_at: Timestamp::now_utc() + AUTH_CODE_VALID_DURATION,
                    })
                    .get_result(db_conn)
                    .await
            })?;

        if application.redirect_uri == SHOW_TOKEN_URI {
            let page = crate::template::render(
                "oauth/token.html",
                minijinja::context! {
                    app_name => application.name,
                    domain => self.url_service.domain(),
                    token => authorization_code.code,
                },
            )
            .unwrap();

            Ok(Html(page).into_response())
        } else {
            let mut url = Url::from_str(&application.redirect_uri)?;
            url.query_pairs_mut()
                .append_pair("code", &authorization_code.code);

            if let Some(state) = state {
                url.query_pairs_mut().append_pair("state", &state);
            }

            Ok(Redirect::to(url.as_str()).into_response())
        }
    }
}
