use super::TOKEN_VALID_DURATION;
use crate::{
    error::{Error, Result},
    state::Zustand,
    util::generate_secret,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use askama::Template;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
    Form,
};
use chrono::Utc;
use http::StatusCode;
use kitsune_db::entity::{oauth2_applications, oauth2_authorization_codes, users};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
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
#[template(path = "authorize.html")]
struct AuthorizePage {
    app_name: String,
    domain: String,
}

#[derive(Template)]
#[template(path = "token.html")]
struct ShowTokenPage {
    app_name: String,
    domain: String,
    token: String,
}

pub async fn get(
    State(state): State<Zustand>,
    Query(query): Query<AuthorizeQuery>,
) -> Result<Response> {
    if query.response_type != "code" {
        return Ok((StatusCode::BAD_REQUEST, "Invalid response type").into_response());
    }

    let application = oauth2_applications::Entity::find_by_id(query.client_id)
        .filter(oauth2_applications::Column::RedirectUri.eq(query.redirect_uri))
        .one(&state.db_conn)
        .await?
        .ok_or(Error::OAuthApplicationNotFound)?;

    let page = AuthorizePage {
        app_name: application.name,
        domain: state.config.domain,
    }
    .render()
    .unwrap();

    Ok(Html(page).into_response())
}

pub async fn post(
    State(state): State<Zustand>,
    Query(query): Query<AuthorizeQuery>,
    Form(form): Form<AuthorizeForm>,
) -> Result<Response> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(form.username))
        .one(&state.db_conn)
        .await?
        .ok_or(Error::UserNotFound)?;

    let application = oauth2_applications::Entity::find_by_id(query.client_id)
        .filter(oauth2_applications::Column::RedirectUri.eq(query.redirect_uri))
        .one(&state.db_conn)
        .await?
        .ok_or(Error::OAuthApplicationNotFound)?;

    let is_valid = crate::blocking::cpu(move || {
        let password_hash = PasswordHash::new(&user.password)?;
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
    .insert(&state.db_conn)
    .await?;

    if application.redirect_uri == SHOW_TOKEN_URI {
        let page = ShowTokenPage {
            app_name: application.name,
            domain: state.config.domain,
            token: authorization_code.code,
        }
        .render()
        .unwrap();

        Ok(Html(page).into_response())
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
