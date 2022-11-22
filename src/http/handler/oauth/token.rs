use crate::{
    db::entity::{
        oauth::{access_token, application, authorization_code, refresh_token},
        user,
    },
    error::{Error, Result},
    http::extractor::OAuthApplication,
    state::State,
    util::generate_secret,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Form, Json,
};
use chrono::{Duration, Utc};
use futures_util::FutureExt;
use once_cell::sync::Lazy;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter, TransactionTrait,
};
use serde::{Deserialize, Serialize};

static ACCESS_TOKEN_VALID_DURATION: Lazy<Duration> = Lazy::new(|| Duration::hours(1));

#[derive(Serialize)]
struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
}

#[derive(Deserialize)]
pub struct AuthorizationCodeData {
    code: String,
    redirect_uri: String,
}

#[derive(Deserialize)]
pub struct PasswordData {
    username: String,
    password: String,
    scope: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenData {
    refresh_token: String,
    scope: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "grant_type")]
pub enum TokenForm {
    AuthorizationCode(AuthorizationCodeData),
    ClientCredentials,
    Password(PasswordData),
    RefreshToken(RefreshTokenData),
}

async fn authorization_code(
    state: State,
    application: application::Model,
    data: AuthorizationCodeData,
) -> Result<Response> {
    let Some((authorization_code, Some(user))) =
        authorization_code::Entity::find_by_id(data.code)
            .find_also_related(user::Entity)
            .one(&state.db_conn)
            .await?
    else {
        return Ok((StatusCode::UNAUTHORIZED, "Unknown authorization code").into_response());
    };

    if application.id != authorization_code.application_id
        || application.redirect_uri != data.redirect_uri
    {
        return Ok((StatusCode::UNAUTHORIZED, "Invalid application credentials").into_response());
    }

    let (access_token, refresh_token) = state
        .db_conn
        .transaction(|tx| {
            async move {
                let access_token = access_token::Model {
                    token: generate_secret(),
                    user_id: Some(user.id),
                    application_id: Some(authorization_code.application_id),
                    created_at: Utc::now(),
                    expired_at: Utc::now() + *ACCESS_TOKEN_VALID_DURATION,
                }
                .into_active_model()
                .insert(tx)
                .await?;

                let refresh_token = refresh_token::Model {
                    token: generate_secret(),
                    access_token: access_token.token.clone(),
                    application_id: application.id,
                    created_at: Utc::now(),
                }
                .into_active_model()
                .insert(tx)
                .await?;

                authorization_code.delete(tx).await?;

                Ok((access_token, refresh_token))
            }
            .boxed()
        })
        .await?;

    Ok(Json(AccessTokenResponse {
        expires_in: access_token.ttl().num_seconds(),
        access_token: access_token.token,
        token_type: "Bearer".into(),
        refresh_token: Some(refresh_token.token),
    })
    .into_response())
}

async fn client_credentials(state: State, application: application::Model) -> Result<Response> {
    let (access_token, refresh_token) = state
        .db_conn
        .transaction(move |tx| {
            async move {
                let access_token = access_token::Model {
                    token: generate_secret(),
                    user_id: None,
                    application_id: Some(application.id),
                    created_at: Utc::now(),
                    expired_at: Utc::now() + *ACCESS_TOKEN_VALID_DURATION,
                }
                .into_active_model()
                .insert(tx)
                .await?;

                let refresh_token = refresh_token::Model {
                    token: generate_secret(),
                    access_token: access_token.token.clone(),
                    application_id: application.id,
                    created_at: Utc::now(),
                }
                .into_active_model()
                .insert(tx)
                .await?;

                Ok((access_token, refresh_token))
            }
            .boxed()
        })
        .await?;

    Ok(Json(AccessTokenResponse {
        expires_in: access_token.ttl().num_seconds(),
        access_token: access_token.token,
        token_type: "Bearer".into(),
        refresh_token: Some(refresh_token.token),
    })
    .into_response())
}

async fn password_grant(state: State, data: PasswordData) -> Result<Response> {
    let user = user::Entity::find()
        .filter(user::Column::Username.eq(data.username))
        .filter(user::Column::Domain.is_null())
        .one(&state.db_conn)
        .await?
        .ok_or(Error::UserNotFound)?;

    let is_valid = crate::blocking::cpu(move || {
        let password = user.password.ok_or(Error::BrokenRecord)?;
        let password_hash = PasswordHash::new(&password)?;
        let argon2 = Argon2::default();

        Ok::<_, Error>(
            argon2
                .verify_password(data.password.as_bytes(), &password_hash)
                .is_ok(),
        )
    })
    .await??;

    if !is_valid {
        return Err(Error::PasswordMismatch);
    }

    let access_token = access_token::Model {
        token: generate_secret(),
        user_id: Some(user.id),
        application_id: None,
        created_at: Utc::now(),
        expired_at: Utc::now() + *ACCESS_TOKEN_VALID_DURATION,
    }
    .into_active_model()
    .insert(&state.db_conn)
    .await?;

    Ok(Json(AccessTokenResponse {
        expires_in: access_token.ttl().num_seconds(),
        access_token: access_token.token,
        token_type: "Bearer".into(),
        refresh_token: None,
    })
    .into_response())
}

async fn refresh_token(
    state: State,
    application: application::Model,
    data: RefreshTokenData,
) -> Result<Response> {
    let Some((refresh_token, Some(access_token))) =
        refresh_token::Entity::find_by_id(data.refresh_token)
            .find_also_related(access_token::Entity)
            .filter(access_token::Column::ApplicationId.is_not_null())
            .one(&state.db_conn)
            .await?
    else {
        return Ok((StatusCode::BAD_REQUEST, "Refresh token not found").into_response());
    };

    if access_token.application_id.unwrap() != application.id {
        return Ok((StatusCode::UNAUTHORIZED, "Invalid application credentials").into_response());
    }

    let (access_token, refresh_token) = state
        .db_conn
        .transaction(|tx| {
            async move {
                let new_access_token = access_token::Model {
                    token: generate_secret(),
                    created_at: Utc::now(),
                    expired_at: Utc::now() + *ACCESS_TOKEN_VALID_DURATION,
                    ..access_token
                }
                .into_active_model()
                .insert(tx)
                .await?;

                let refresh_token = refresh_token::ActiveModel {
                    token: ActiveValue::Set(refresh_token.token),
                    access_token: ActiveValue::Set(new_access_token.token.clone()),
                    ..Default::default()
                }
                .update(tx)
                .await?;

                access_token::Entity::delete_by_id(access_token.token)
                    .exec(tx)
                    .await?;

                Ok((new_access_token, refresh_token))
            }
            .boxed()
        })
        .await?;

    Ok(Json(AccessTokenResponse {
        expires_in: access_token.ttl().num_seconds(),
        access_token: access_token.token,
        token_type: "Bearer".into(),
        refresh_token: Some(refresh_token.token),
    })
    .into_response())
}

pub async fn post(
    Extension(state): Extension<State>,
    application: Option<OAuthApplication>,
    Form(form): Form<TokenForm>,
) -> Result<Response> {
    match (form, application) {
        (TokenForm::AuthorizationCode(data), Some(OAuthApplication(application))) => {
            authorization_code(state, application, data).await
        }
        (TokenForm::ClientCredentials, Some(OAuthApplication(application))) => {
            client_credentials(state, application).await
        }
        (TokenForm::Password(data), ..) => password_grant(state, data).await,
        (TokenForm::RefreshToken(data), Some(OAuthApplication(application))) => {
            refresh_token(state, application, data).await
        }
        (
            TokenForm::AuthorizationCode(..)
            | TokenForm::ClientCredentials
            | TokenForm::RefreshToken(..),
            None,
        ) => Ok((
            StatusCode::UNAUTHORIZED,
            "Missing OAuth application credentials",
        )
            .into_response()),
    }
}
