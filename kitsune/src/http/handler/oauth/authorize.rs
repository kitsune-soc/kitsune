use crate::{
    error::{Error, Result},
    service::oauth2::{Oauth2Service, OauthEndpoint, OauthOwnerSolicitor},
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use askama::Template;
use axum::{
    extract::{Query, State},
    response::Response,
    Form,
};
use axum_extra::extract::SignedCookieJar;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{oauth2, user::User},
    schema::{oauth2_applications, users},
    PgPool,
};
use oxide_auth_async::endpoint::authorization::AuthorizationFlow;
use oxide_auth_axum::{OAuthRequest, OAuthResponse};
use serde::Deserialize;
use uuid::Uuid;

#[cfg(feature = "oidc")]
use {crate::service::oidc::OidcService, axum::response::Redirect};

#[derive(Deserialize)]
pub struct AuthorizeQuery {
    response_type: String,
    client_id: Uuid,
    redirect_uri: String,
    state: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[derive(Template)]
#[template(path = "oauth/login.html")]
struct LoginPage {
    app_name: String,
    domain: String,
}

pub async fn get(
    State(db_conn): State<PgPool>,
    #[cfg(feature = "oidc")] State(oidc_service): State<Option<OidcService>>,
    State(oauth_endpoint): State<OauthEndpoint>,
    cookies: SignedCookieJar,
    oauth_req: OAuthRequest,
) -> Result<OAuthResponse> {
    #[cfg(feature = "oidc")]
    #[allow(clippy::used_underscore_binding)]
    if let Some(oidc_service) = oidc_service {
        let mut db_conn = db_conn.get().await?;
        let application = oauth2_applications::table
            .find(query.client_id)
            .filter(oauth2_applications::redirect_uri.eq(query.redirect_uri))
            .get_result::<oauth2::Application>(&mut db_conn)
            .await?;

        let auth_url = oidc_service
            .authorisation_url(application.id, query.state)
            .await?;

        return Ok(Redirect::to(auth_url.as_str()).into_response());
    }

    let solicitor = OauthOwnerSolicitor::builder().db_pool(db_conn).build();
    let mut flow = AuthorizationFlow::prepare(oauth_endpoint.with_solicitor(solicitor))?;
    AuthorizationFlow::execute(&mut flow, oauth_req)
        .await
        .map_err(Error::from)
}

pub async fn post(
    State(db_conn): State<PgPool>,
    State(oauth2_service): State<Oauth2Service>,
    Query(query): Query<AuthorizeQuery>,
    Form(form): Form<LoginForm>,
) -> Result<Response> {
    let mut db_conn = db_conn.get().await?;
    let user = users::table
        .filter(users::username.eq(form.username))
        .first::<User>(&mut db_conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => Error::PasswordMismatch,
            e => e.into(),
        })?;

    let application = oauth2_applications::table
        .find(query.client_id)
        .filter(oauth2_applications::redirect_uri.eq(query.redirect_uri))
        .get_result::<oauth2::Application>(&mut db_conn)
        .await?;

    let is_valid = crate::blocking::cpu(move || {
        let password_hash = PasswordHash::new(user.password.as_ref().unwrap())?;
        let argon2 = Argon2::default();

        Ok::<_, Error>(
            argon2
                .verify_password(form.password.as_bytes(), &password_hash)
                .is_ok(),
        )
    })
    .await??;

    if !is_valid {
        return Err(Error::PasswordMismatch);
    }

    /*let authorisation_code = AuthorisationCode::builder()
        .application(application)
        .state(query.state)
        .user_id(user.id)
        .build();

    oauth2_service
        .create_authorisation_code_response(authorisation_code)
        .await*/
    todo!();
}
