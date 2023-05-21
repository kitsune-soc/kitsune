use crate::{
    error::{Error, Result},
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
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{
    scoped_futures::ScopedFutureExt, AsyncConnection, AsyncPgConnection, RunQueryDsl,
};
use http::StatusCode;
use kitsune_db::{
    function::now,
    model::{oauth2, user::User},
    schema::{
        oauth2_access_tokens, oauth2_applications, oauth2_authorization_codes,
        oauth2_refresh_tokens, users,
    },
    PgPool,
};
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

async fn get_application(
    db_conn: &mut AsyncPgConnection,
    id: Uuid,
    secret: String,
    redirect_uri: Option<String>,
) -> Result<oauth2::Application> {
    let mut query = oauth2_applications::table
        .find(id)
        .filter(oauth2_applications::secret.eq(secret))
        .into_boxed();

    if let Some(redirect_uri) = redirect_uri {
        query = query.filter(oauth2_applications::redirect_uri.eq(redirect_uri));
    }

    query.get_result(db_conn).await.map_err(Error::from)
}

async fn authorization_code(
    db_conn: &mut AsyncPgConnection,
    data: AuthorizationCodeData,
) -> Result<Response> {
    let Some((authorization_code, user)) = oauth2_authorization_codes::table
        .find(data.code)
        .filter(oauth2_authorization_codes::expired_at.gt(now()))
        .inner_join(users::table)
        .get_result::<(oauth2::AuthorizationCode, User)>(db_conn)
        .await
        .optional()?
    else {
        return Ok((StatusCode::UNAUTHORIZED, "Unknown authorization code").into_response());
    };

    let application = get_application(
        db_conn,
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
                let access_token = diesel::insert_into(oauth2_access_tokens::table)
                    .values(oauth2::NewAccessToken {
                        token: generate_secret().as_str(),
                        user_id: Some(user.id),
                        application_id: Some(authorization_code.application_id),
                        expired_at: OffsetDateTime::now_utc() + TOKEN_VALID_DURATION,
                    })
                    .get_result::<oauth2::AccessToken>(tx)
                    .await?;

                let refresh_token = diesel::insert_into(oauth2_refresh_tokens::table)
                    .values(oauth2::NewRefreshToken {
                        token: generate_secret().as_str(),
                        access_token: access_token.token.as_str(),
                        application_id: application.id,
                    })
                    .get_result::<oauth2::RefreshToken>(tx)
                    .await?;

                diesel::delete(&authorization_code).execute(tx).await?;

                Ok::<_, Error>((access_token, refresh_token))
            }
            .scope_boxed()
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
    db_conn: &mut AsyncPgConnection,
    data: ClientCredentialsData,
) -> Result<Response> {
    let (access_token, refresh_token) = db_conn
        .transaction(move |tx| {
            async move {
                let application =
                    get_application(tx, data.client_id, data.client_secret, None).await?;

                let access_token = diesel::insert_into(oauth2_access_tokens::table)
                    .values(oauth2::NewAccessToken {
                        token: generate_secret().as_str(),
                        user_id: None,
                        application_id: Some(application.id),
                        expired_at: OffsetDateTime::now_utc() + TOKEN_VALID_DURATION,
                    })
                    .get_result::<oauth2::AccessToken>(tx)
                    .await?;

                let refresh_token = diesel::insert_into(oauth2_refresh_tokens::table)
                    .values(oauth2::NewRefreshToken {
                        token: generate_secret().as_str(),
                        access_token: access_token.token.as_str(),
                        application_id: application.id,
                    })
                    .get_result::<oauth2::RefreshToken>(tx)
                    .await?;

                Ok::<_, Error>((access_token, refresh_token))
            }
            .scope_boxed()
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

async fn password_grant(db_conn: &mut AsyncPgConnection, data: PasswordData) -> Result<Response> {
    let user = users::table
        .filter(users::username.eq(data.username))
        .first::<User>(db_conn)
        .await?;

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

    let access_token = diesel::insert_into(oauth2_access_tokens::table)
        .values(oauth2::NewAccessToken {
            token: generate_secret().as_str(),
            user_id: Some(user.id),
            application_id: None,
            expired_at: OffsetDateTime::now_utc() + TOKEN_VALID_DURATION,
        })
        .get_result::<oauth2::AccessToken>(db_conn)
        .await?;

    Ok(Json(AccessTokenResponse {
        expires_in: access_token.ttl().whole_seconds(),
        access_token: access_token.token,
        token_type: "Bearer".into(),
        refresh_token: None,
    })
    .into_response())
}

async fn refresh_token(
    db_conn: &mut AsyncPgConnection,
    data: RefreshTokenData,
) -> Result<Response> {
    let Some((refresh_token, access_token)) = oauth2_refresh_tokens::table
        .find(data.refresh_token)
        .inner_join(oauth2_access_tokens::table)
        .filter(oauth2_access_tokens::application_id.is_not_null())
        .get_result::<(oauth2::RefreshToken, oauth2::AccessToken)>(db_conn)
        .await
        .optional()?
    else {
        return Ok((StatusCode::BAD_REQUEST, "Refresh token not found").into_response());
    };

    let application = get_application(db_conn, data.client_id, data.client_secret, None).await?;
    if access_token.application_id.unwrap() != application.id {
        return Ok((StatusCode::UNAUTHORIZED, "Invalid application credentials").into_response());
    }

    let (access_token, refresh_token) = db_conn
        .transaction(|tx| {
            async move {
                let new_access_token = diesel::insert_into(oauth2_access_tokens::table)
                    .values(oauth2::NewAccessToken {
                        user_id: access_token.user_id,
                        token: generate_secret().as_str(),
                        application_id: access_token.application_id,
                        expired_at: OffsetDateTime::now_utc() + TOKEN_VALID_DURATION,
                    })
                    .get_result::<oauth2::AccessToken>(tx)
                    .await?;

                let refresh_token = diesel::update(&refresh_token)
                    .set(oauth2_refresh_tokens::access_token.eq(new_access_token.token.as_str()))
                    .get_result::<oauth2::RefreshToken>(tx)
                    .await?;

                diesel::delete(&access_token).execute(tx).await?;

                Ok::<_, Error>((new_access_token, refresh_token))
            }
            .scope_boxed()
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
    State(db_conn): State<PgPool>,
    FormOrJson(form): FormOrJson<TokenForm>,
) -> Result<Response> {
    let mut db_conn = db_conn.get().await?;

    match form {
        TokenForm::AuthorizationCode(data) => authorization_code(&mut db_conn, data).await,
        TokenForm::ClientCredentials(data) => client_credentials(&mut db_conn, data).await,
        TokenForm::Password(data) => password_grant(&mut db_conn, data).await,
        TokenForm::RefreshToken(data) => refresh_token(&mut db_conn, data).await,
    }
}
