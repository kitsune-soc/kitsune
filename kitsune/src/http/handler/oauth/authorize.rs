use crate::{
    error::{Error, Result},
    service::{
        oauth2::{AuthorisationCode, Oauth2Service},
        oidc::OidcService,
        url::UrlService,
    },
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Form,
};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use http::StatusCode;
use kitsune_db::{
    model::{oauth2, user::User},
    schema::{oauth2_applications, users},
    PgPool,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AuthorizeQuery {
    response_type: String,
    client_id: Uuid,
    redirect_uri: String,
    state: Option<String>,
}

#[derive(Deserialize)]
pub struct AuthorizeForm {
    username: String,
    password: String,
}

#[derive(Template)]
#[template(path = "oauth/authorize.html")]
struct AuthorizePage {
    app_name: String,
    domain: String,
}

pub async fn get(
    State(db_conn): State<PgPool>,
    State(oidc_service): State<Option<OidcService>>,
    State(url_service): State<UrlService>,
    Query(query): Query<AuthorizeQuery>,
) -> Result<Response> {
    if query.response_type != "code" {
        return Ok((StatusCode::BAD_REQUEST, "Invalid response type").into_response());
    }

    let mut db_conn = db_conn.get().await?;
    let application = oauth2_applications::table
        .find(query.client_id)
        .filter(oauth2_applications::redirect_uri.eq(query.redirect_uri))
        .get_result::<oauth2::Application>(&mut db_conn)
        .await?;

    if let Some(oidc_service) = oidc_service {
        let auth_url = oidc_service
            .authorisation_url(application.id, query.state)
            .await?;

        Ok((StatusCode::FOUND, [("Location", auth_url.as_str())]).into_response())
    } else {
        Ok(AuthorizePage {
            app_name: application.name,
            domain: url_service.domain().into(),
        }
        .into_response())
    }
}

pub async fn post(
    State(db_conn): State<PgPool>,
    State(oauth2_service): State<Oauth2Service>,
    Query(query): Query<AuthorizeQuery>,
    Form(form): Form<AuthorizeForm>,
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

    let authorisation_code = AuthorisationCode::builder()
        .application(application)
        .state(query.state)
        .user_id(user.id)
        .build();

    oauth2_service
        .create_authorisation_code_response(authorisation_code)
        .await
}
