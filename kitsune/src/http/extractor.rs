use crate::{
    db::model::{account, oauth::access_token, user},
    error::Error,
    state::Zustand,
};
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequest, FromRequestParts},
    response::{IntoResponse, Response},
    Form, Json, RequestExt, RequestPartsExt, TypedHeader,
};
use chrono::Utc;
use headers::{authorization::Bearer, Authorization, ContentType};
use http::{request::Parts, StatusCode};
use kitsune_http_signatures::{
    ring::signature::{UnparsedPublicKey, RSA_PKCS1_2048_8192_SHA256},
    HttpVerifier,
};
use kitsune_type::ap::Activity;
use mime::Mime;
use rsa::pkcs8::{Document, SubjectPublicKeyInfo};
use sea_orm::{ColumnTrait, QueryFilter, Related};
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct UserData {
    pub account: account::Model,
    pub user: user::Model,
}

pub struct AuthExtactor(pub UserData);

#[async_trait]
impl FromRequestParts<Zustand> for AuthExtactor {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Zustand,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization::<Bearer>(bearer_token)) = parts
            .extract_with_state(state)
            .await
            .map_err(IntoResponse::into_response)?;

        let Some((user, Some(account))) =
            <access_token::Entity as Related<user::Entity>>::find_related()
                .filter(access_token::Column::Token.eq(bearer_token.token()))
                .filter(access_token::Column::ExpiredAt.gt(Utc::now()))
                .find_also_related(account::Entity)
                .one(&state.db_conn)
                .await
                .map_err(Error::from)?
        else {
            return Err(StatusCode::UNAUTHORIZED.into_response());
        };

        Ok(Self(UserData { account, user }))
    }
}

pub struct FormOrJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S, Body> for FormOrJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send + 'static,
{
    type Rejection = Response;

    async fn from_request(
        mut req: http::Request<Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(content_type) = req
            .extract_parts::<TypedHeader<ContentType>>()
            .await
            .map_err(IntoResponse::into_response)?;

        let content = if Mime::from(content_type)
            .as_ref()
            .starts_with(mime::APPLICATION_WWW_FORM_URLENCODED.as_ref())
        {
            Form::from_request(req, state)
                .await
                .map_err(IntoResponse::into_response)?
                .0
        } else {
            Json::from_request(req, state)
                .await
                .map_err(IntoResponse::into_response)?
                .0
        };

        Ok(Self(content))
    }
}

pub struct SignedActivity(pub Activity);

#[async_trait]
impl FromRequest<Zustand, Body> for SignedActivity {
    type Rejection = Response;

    async fn from_request(
        req: http::Request<Body>,
        state: &Zustand,
    ) -> Result<Self, Self::Rejection> {
        let (parts, body) = req
            .with_limited_body()
            .expect("[Bug] Payload size of inbox not limited")
            .into_parts();

        let activity: Activity = match hyper::body::to_bytes(body).await {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(Error::from)?,
            Err(err) => {
                debug!(error = %err, "Failed to buffer body");
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        };
        let ap_id = activity
            .rest
            .attributed_to()
            .ok_or_else(|| StatusCode::BAD_REQUEST.into_response())?;

        let remote_user = state.fetcher.fetch_actor(ap_id).await?;
        let (_tag, public_key) =
            Document::from_pem(&remote_user.public_key).map_err(Error::from)?;
        let public_key: SubjectPublicKeyInfo<'_> = public_key.decode_msg().map_err(Error::from)?;
        let public_key = UnparsedPublicKey::new(
            &RSA_PKCS1_2048_8192_SHA256,
            public_key.subject_public_key.to_vec(),
        );

        let is_valid = HttpVerifier::builder()
            .build()
            .unwrap()
            .verify(&parts, |_key_id| async move {
                // TODO: Select from the database by key ID
                Ok(public_key)
            })
            .await
            .is_ok();

        if !is_valid {
            return Err(StatusCode::UNAUTHORIZED.into_response());
        }

        Ok(Self(activity))
    }
}
