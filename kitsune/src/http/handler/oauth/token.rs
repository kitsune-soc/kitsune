use crate::{
    error::{ApiError, Error, Result},
    http::extractor::FormOrJson,
    service::oauth2::TOKEN_VALID_DURATION,
    util::{generate_secret, AccessTokenTtl},
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use futures_util::FutureExt;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct AuthorizationCodeData {
    client_id: Uuid,
    client_secret: String,
    code: String,
    redirect_uri: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ClientCredentialsData {
    client_id: Uuid,
    client_secret: String,
}

#[derive(Deserialize, ToSchema)]
pub struct PasswordData {
    username: String,
    #[schema(format = Password)]
    password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshTokenData {
    client_id: Uuid,
    client_secret: String,
    refresh_token: String,
}

#[derive(Deserialize, ToSchema)]
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
) -> Result<oauth2_applications::Model>
where
    C: ConnectionTrait,
{
    let mut query =
        Oauth2Applications::find_by_id(id).filter(oauth2_applications::Column::Secret.eq(secret));
    if let Some(redirect_uri) = redirect_uri {
        query = query.filter(oauth2_applications::Column::RedirectUri.eq(redirect_uri));
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
        Oauth2AuthorizationCodes::find_by_id(data.code)
            .filter(oauth2_authorization_codes::Column::ExpiredAt.gt(OffsetDateTime::now_utc()))
            .find_also_related(Users)
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
                let access_token = oauth2_access_tokens::Model {
                    token: generate_secret(),
                    user_id: Some(user.id),
                    application_id: Some(authorization_code.application_id),
                    created_at: OffsetDateTime::now_utc(),
                    expired_at: OffsetDateTime::now_utc() + TOKEN_VALID_DURATION,
                }
                .into_active_model()
                .insert(tx)
                .await?;

                let refresh_token = oauth2_refresh_tokens::Model {
                    token: generate_secret(),
                    access_token: access_token.token.clone(),
                    application_id: application.id,
                    created_at: OffsetDateTime::now_utc(),
                }
                .into_active_model()
                .insert(tx)
                .await?;

                authorization_code.delete(tx).await?;

                Ok::<_, Error>((access_token, refresh_token))
            }
            .boxed()
        })
        .await?;

    Ok(Json(AccessTokenResponse {
        expires_in: access_token.ttl().whole_seconds(),
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

                let access_token = oauth2_access_tokens::Model {
                    token: generate_secret(),
                    user_id: None,
                    application_id: Some(application.id),
                    created_at: OffsetDateTime::now_utc(),
                    expired_at: OffsetDateTime::now_utc() + TOKEN_VALID_DURATION,
                }
                .into_active_model()
                .insert(tx)
                .await?;

                let refresh_token = oauth2_refresh_tokens::Model {
                    token: generate_secret(),
                    access_token: access_token.token.clone(),
                    application_id: application.id,
                    created_at: OffsetDateTime::now_utc(),
                }
                .into_active_model()
                .insert(tx)
                .await?;

                Ok::<_, Error>((access_token, refresh_token))
            }
            .boxed()
        })
        .await?;

    Ok(Json(AccessTokenResponse {
        expires_in: access_token.ttl().whole_seconds(),
        access_token: access_token.token,
        token_type: "Bearer".into(),
        refresh_token: Some(refresh_token.token),
    })
    .into_response())
}

async fn password_grant(db_conn: DatabaseConnection, data: PasswordData) -> Result<Response> {
    let user = Users::find()
        .filter(users::Column::Username.eq(data.username))
        .one(&db_conn)
        .await?
        .ok_or(ApiError::NotFound)?;

    let is_valid = crate::blocking::cpu(move || {
        let password_hash = PasswordHash::new(user.password.as_ref().unwrap())?;
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

    let access_token = oauth2_access_tokens::Model {
        token: generate_secret(),
        user_id: Some(user.id),
        application_id: None,
        created_at: OffsetDateTime::now_utc(),
        expired_at: OffsetDateTime::now_utc() + TOKEN_VALID_DURATION,
    }
    .into_active_model()
    .insert(&db_conn)
    .await?;

    Ok(Json(AccessTokenResponse {
        expires_in: access_token.ttl().whole_seconds(),
        access_token: access_token.token,
        token_type: "Bearer".into(),
        refresh_token: None,
    })
    .into_response())
}

async fn refresh_token(db_conn: DatabaseConnection, data: RefreshTokenData) -> Result<Response> {
    let Some((refresh_token, Some(access_token))) =
        Oauth2RefreshTokens::find_by_id(data.refresh_token)
            .filter(oauth2_access_tokens::Column::ApplicationId.is_not_null())
            .find_also_related(Oauth2AccessTokens)
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
                let new_access_token = oauth2_access_tokens::Model {
                    token: generate_secret(),
                    created_at: OffsetDateTime::now_utc(),
                    expired_at: OffsetDateTime::now_utc() + TOKEN_VALID_DURATION,
                    ..access_token
                }
                .into_active_model()
                .insert(tx)
                .await?;

                let refresh_token = oauth2_refresh_tokens::ActiveModel {
                    token: ActiveValue::Set(refresh_token.token),
                    access_token: ActiveValue::Set(new_access_token.token.clone()),
                    ..Default::default()
                }
                .update(tx)
                .await?;

                Oauth2AccessTokens::delete_by_id(access_token.token)
                    .exec(tx)
                    .await?;

                Ok::<_, Error>((new_access_token, refresh_token))
            }
            .boxed()
        })
        .await?;

    Ok(Json(AccessTokenResponse {
        expires_in: access_token.ttl().whole_seconds(),
        access_token: access_token.token,
        token_type: "Bearer".into(),
        refresh_token: Some(refresh_token.token),
    })
    .into_response())
}

#[utoipa::path(
    post,
    path = "/oauth/token",
    request_body = TokenForm,
    responses(
        (status = 200, description = "Newly created token", body = AccessTokenResponse)
    )
)]
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
