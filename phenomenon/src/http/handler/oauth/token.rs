use super::TOKEN_VALID_DURATION;
use crate::{
    db::model::{
        oauth::{access_token, application, authorization_code, refresh_token},
        user,
    },
    error::{Error, Result},
    http::extractor::FormOrJson,
    util::generate_secret,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use futures_util::FutureExt;
use http::StatusCode;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, ModelTrait, QueryFilter, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    client_id: Uuid,
    client_secret: String,
    code: String,
    redirect_uri: String,
}

#[derive(Deserialize)]
pub struct ClientCredentialsData {
    client_id: Uuid,
    client_secret: String,
}

#[derive(Deserialize)]
pub struct PasswordData {
    username: String,
    password: String,
    scope: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenData {
    client_id: Uuid,
    client_secret: String,
    refresh_token: String,
    scope: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "grant_type")]
pub enum TokenForm {
    AuthorizationCode(AuthorizationCodeData),
    ClientCredentials(ClientCredentialsData),
    Password(PasswordData),
    RefreshToken(RefreshTokenData),
}

async fn get_application<C>(
    db_conn: &C,
    id: Uuid,
    secret: String,
    redirect_uri: Option<String>,
) -> Result<application::Model>
where
    C: ConnectionTrait,
{
    let mut query =
        application::Entity::find_by_id(id).filter(application::Column::Secret.eq(secret));
    if let Some(redirect_uri) = redirect_uri {
        query = query.filter(application::Column::RedirectUri.eq(redirect_uri));
    }

    query
        .one(db_conn)
        .await?
        .ok_or(Error::OAuthApplicationNotFound)
}

async fn authorization_code(
    db_conn: DatabaseConnection,
    data: AuthorizationCodeData,
) -> Result<Response> {
    let Some((authorization_code, Some(user))) =
        authorization_code::Entity::find_by_id(data.code)
            .filter(authorization_code::Column::ExpiredAt.gt(Utc::now()))
            .find_also_related(user::Entity)
            .one(&db_conn)
            .await?
    else {
        return Ok((StatusCode::UNAUTHORIZED, "Unknown authorization code").into_response());
    };

    let application = get_application(
        &db_conn,
        data.client_id,
        data.client_secret,
        Some(data.redirect_uri),
    )
    .await?;

    if application.id != authorization_code.application_id {
        return Ok((StatusCode::UNAUTHORIZED, "Invalid application credentials").into_response());
    }

    let (access_token, refresh_token) = db_conn
        .transaction(|tx| {
            async move {
                let access_token = access_token::Model {
                    token: generate_secret(),
                    user_id: Some(user.id),
                    application_id: Some(authorization_code.application_id),
                    created_at: Utc::now(),
                    expired_at: Utc::now() + *TOKEN_VALID_DURATION,
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

async fn client_credentials(
    db_conn: DatabaseConnection,
    data: ClientCredentialsData,
) -> Result<Response> {
    let (access_token, refresh_token) = db_conn
        .transaction(move |tx| {
            async move {
                let application =
                    get_application(tx, data.client_id, data.client_secret, None).await?;

                let access_token = access_token::Model {
                    token: generate_secret(),
                    user_id: None,
                    application_id: Some(application.id),
                    created_at: Utc::now(),
                    expired_at: Utc::now() + *TOKEN_VALID_DURATION,
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

async fn password_grant(db_conn: DatabaseConnection, data: PasswordData) -> Result<Response> {
    let user = user::Entity::find()
        .filter(
            user::Column::Username
                .eq(data.username)
                .and(user::Column::Domain.is_null()),
        )
        .one(&db_conn)
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
        expired_at: Utc::now() + *TOKEN_VALID_DURATION,
    }
    .into_active_model()
    .insert(&db_conn)
    .await?;

    Ok(Json(AccessTokenResponse {
        expires_in: access_token.ttl().num_seconds(),
        access_token: access_token.token,
        token_type: "Bearer".into(),
        refresh_token: None,
    })
    .into_response())
}

async fn refresh_token(db_conn: DatabaseConnection, data: RefreshTokenData) -> Result<Response> {
    let Some((refresh_token, Some(access_token))) =
        refresh_token::Entity::find_by_id(data.refresh_token)
            .filter(access_token::Column::ApplicationId.is_not_null())
            .find_also_related(access_token::Entity)
            .one(&db_conn)
            .await?
    else {
        return Ok((StatusCode::BAD_REQUEST, "Refresh token not found").into_response());
    };

    let application = get_application(&db_conn, data.client_id, data.client_secret, None).await?;
    if access_token.application_id.unwrap() != application.id {
        return Ok((StatusCode::UNAUTHORIZED, "Invalid application credentials").into_response());
    }

    let (access_token, refresh_token) = db_conn
        .transaction(|tx| {
            async move {
                let new_access_token = access_token::Model {
                    token: generate_secret(),
                    created_at: Utc::now(),
                    expired_at: Utc::now() + *TOKEN_VALID_DURATION,
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
    State(db_conn): State<DatabaseConnection>,
    FormOrJson(form): FormOrJson<TokenForm>,
) -> Result<Response> {
    match form {
        TokenForm::AuthorizationCode(data) => authorization_code(db_conn, data).await,
        TokenForm::ClientCredentials(data) => client_credentials(db_conn, data).await,
        TokenForm::Password(data) => password_grant(db_conn, data).await,
        TokenForm::RefreshToken(data) => refresh_token(db_conn, data).await,
    }
}
