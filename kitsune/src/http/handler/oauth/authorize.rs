use crate::{
    error::{Error, Result},
    service::oauth2::{OauthEndpoint, OauthOwnerSolicitor},
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use askama::Template;
use axum::{
    debug_handler,
    extract::{OriginalUri, State},
    response::Redirect,
    Form,
};
use axum_extra::{
    either::{Either, Either3},
    extract::{
        cookie::{Cookie, Expiration, SameSite},
        SignedCookieJar,
    },
};
use axum_flash::{Flash, IncomingFlashes};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{model::user::User, schema::users, PgPool};
use oxide_auth_async::endpoint::authorization::AuthorizationFlow;
use oxide_auth_axum::{OAuthRequest, OAuthResponse};
use serde::Deserialize;
use uuid::Uuid;

#[cfg(feature = "oidc")]
use {
    crate::service::oidc::OidcService,
    axum::extract::Query,
    kitsune_db::{model::oauth2, schema::oauth2_applications},
};

#[cfg(feature = "oidc")]
#[derive(Deserialize)]
pub struct AuthorizeQuery {
    client_id: Uuid,
    redirect_uri: String,
    scope: String,
    state: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[derive(Template)]
#[template(path = "oauth/login.html")]
pub struct LoginPage {
    flash_messages: IncomingFlashes,
}

pub async fn get(
    #[cfg(feature = "oidc")] State(oidc_service): State<Option<OidcService>>,
    #[cfg(feature = "oidc")] Query(query): Query<AuthorizeQuery>,
    State(db_pool): State<PgPool>,
    State(oauth_endpoint): State<OauthEndpoint>,
    cookies: SignedCookieJar,
    flash_messages: IncomingFlashes,
    oauth_req: OAuthRequest,
) -> Result<Either3<OAuthResponse, LoginPage, Redirect>> {
    #[cfg(feature = "oidc")]
    if let Some(oidc_service) = oidc_service {
        let mut db_conn = db_pool.get().await?;
        let application = oauth2_applications::table
            .find(query.client_id)
            .filter(oauth2_applications::redirect_uri.eq(query.redirect_uri))
            .get_result::<oauth2::Application>(&mut db_conn)
            .await?;

        let auth_url = oidc_service
            .authorisation_url(application.id, query.scope, query.state)
            .await?;

        return Ok(Either3::E3(Redirect::to(auth_url.as_str())));
    }

    let authenticated_user = if let Some(user_id) = cookies.get("user_id") {
        let mut db_conn = db_pool.get().await?;
        let id = user_id.value().parse::<Uuid>()?;
        users::table.find(id).get_result(&mut db_conn).await?
    } else {
        return Ok(Either3::E2(LoginPage { flash_messages }));
    };

    let solicitor = OauthOwnerSolicitor::builder()
        .authenticated_user(authenticated_user)
        .db_pool(db_pool)
        .build();

    let mut flow = AuthorizationFlow::prepare(oauth_endpoint.with_solicitor(solicitor))?;
    AuthorizationFlow::execute(&mut flow, oauth_req)
        .await
        .map(Either3::E1)
        .map_err(Error::from)
}

#[debug_handler(state = crate::state::Zustand)]
pub async fn post(
    State(db_conn): State<PgPool>,
    OriginalUri(original_url): OriginalUri,
    cookies: SignedCookieJar,
    flash: Flash,
    Form(form): Form<LoginForm>,
) -> Result<Either<(SignedCookieJar, Redirect), (Flash, Redirect)>> {
    let redirect_to = if let Some(path_and_query) = original_url.path_and_query() {
        path_and_query.as_str()
    } else {
        original_url.path()
    };

    let mut db_conn = db_conn.get().await?;
    let Some(user) = users::table
        .filter(users::username.eq(form.username))
        .first::<User>(&mut db_conn)
        .await
        .optional()?
    else {
        return Ok(
            Either::E2(
                (flash.error(Error::PasswordMismatch.to_string()), Redirect::to(redirect_to))
            )
        );
    };

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
        return Ok(Either::E2((
            flash.error(Error::PasswordMismatch.to_string()),
            Redirect::to(redirect_to),
        )));
    }

    let user_id_cookie = Cookie::build("user_id", user.id.to_string())
        .secure(true)
        .same_site(SameSite::Strict)
        .expires(Expiration::Session)
        .finish();

    Ok(Either::E1((
        cookies.add(user_id_cookie),
        Redirect::to(redirect_to),
    )))
}
