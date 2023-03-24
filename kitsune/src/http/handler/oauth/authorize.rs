use super::TOKEN_VALID_DURATION;
use crate::{
    error::{ApiError, Error, Result},
    service::{oidc::OidcService, url::UrlService},
    util::generate_secret,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Form,
};
use chrono::Utc;
use http::StatusCode;
use kitsune_db::entity::{
    oauth2_applications, oauth2_authorization_codes,
    prelude::{Oauth2Applications, Users},
    users,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
};
use serde::Deserialize;
use std::str::FromStr;
use url::Url;
use uuid::Uuid;

/// If the Redirect URI is equal to this string, show the token instead of redirecting the user
const SHOW_TOKEN_URI: &str = "urn:ietf:wg:oauth:2.0:oob";

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

#[derive(Template)]
#[template(path = "oauth/token.html")]
struct ShowTokenPage {
    app_name: String,
    domain: String,
    token: String,
}

pub async fn get(
    State(db_conn): State<DatabaseConnection>,
    State(oidc_service): State<Option<OidcService>>,
    State(url_service): State<UrlService>,
    Query(query): Query<AuthorizeQuery>,
) -> Result<Response> {
    if let Some(oidc_service) = oidc_service {
        let auth_url = oidc_service.authorisation_url().await?;
        return Ok((StatusCode::FOUND, [("Location", auth_url.as_str())]).into_response());
    }

    if query.response_type != "code" {
        return Ok((StatusCode::BAD_REQUEST, "Invalid response type").into_response());
    }

    let application = Oauth2Applications::find_by_id(query.client_id)
        .filter(oauth2_applications::Column::RedirectUri.eq(query.redirect_uri))
        .one(&db_conn)
        .await?
        .ok_or(Error::OAuthApplicationNotFound)?;

    Ok(AuthorizePage {
        app_name: application.name,
        domain: url_service.domain().into(),
    }
    .into_response())
}

pub async fn post(
    State(db_conn): State<DatabaseConnection>,
    State(url_service): State<UrlService>,
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

    let authorization_code = oauth2_authorization_codes::Model {
        code: generate_secret(),
        application_id: application.id,
        user_id: user.id,
        created_at: Utc::now().into(),
        expired_at: (Utc::now() + *TOKEN_VALID_DURATION).into(),
    }
    .into_active_model()
    .insert(&db_conn)
    .await?;

    if application.redirect_uri == SHOW_TOKEN_URI {
        Ok(ShowTokenPage {
            app_name: application.name,
            domain: url_service.domain().into(),
            token: authorization_code.code,
        }
        .into_response())
    } else {
        let mut url = Url::from_str(&application.redirect_uri)?;
        url.query_pairs_mut()
            .append_pair("code", &authorization_code.code);

        if let Some(state) = query.state {
            url.query_pairs_mut().append_pair("state", &state);
        }

        Ok((StatusCode::FOUND, [("Location", url.as_str())]).into_response())
    }
}
