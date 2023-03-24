use crate::{
    error::{ApiError, Error, Result},
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
use http::StatusCode;
use kitsune_db::entity::{
    oauth2_applications,
    prelude::{Oauth2Applications, Users},
    users,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
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
    State(db_conn): State<DatabaseConnection>,
    State(oidc_service): State<Option<OidcService>>,
    State(url_service): State<UrlService>,
    Query(query): Query<AuthorizeQuery>,
) -> Result<Response> {
    if query.response_type != "code" {
        return Ok((StatusCode::BAD_REQUEST, "Invalid response type").into_response());
    }

    let application = Oauth2Applications::find_by_id(query.client_id)
        .filter(oauth2_applications::Column::RedirectUri.eq(query.redirect_uri))
        .one(&db_conn)
        .await?
        .ok_or(Error::OAuthApplicationNotFound)?;

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
    State(db_conn): State<DatabaseConnection>,
    State(oauth2_service): State<Oauth2Service>,
    Query(query): Query<AuthorizeQuery>,
    Form(form): Form<AuthorizeForm>,
) -> Result<Response> {
    let user = Users::find()
        .filter(users::Column::Username.eq(form.username))
        .one(&db_conn)
        .await?
        .ok_or(ApiError::NotFound)?;

    let application = Oauth2Applications::find_by_id(query.client_id)
        .filter(oauth2_applications::Column::RedirectUri.eq(query.redirect_uri))
        .one(&db_conn)
        .await?
        .ok_or(Error::OAuthApplicationNotFound)?;

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
        .build()
        .unwrap();

    oauth2_service
        .create_authorisation_code_response(authorisation_code)
        .await
}
