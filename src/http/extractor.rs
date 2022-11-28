use crate::{
    db::entity::{oauth::access_token, user},
    error::Error,
    state::State,
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
use phenomenon_http_signatures::Request;
use phenomenon_model::ap::Activity;
use rsa::pkcs1::EncodeRsaPublicKey;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::de::DeserializeOwned;

pub struct AuthExtactor(pub Option<user::Model>);

#[async_trait]
impl FromRequestParts<State> for AuthExtactor {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        if let Ok(TypedHeader(Authorization::<Bearer>(bearer_token))) =
            parts.extract_with_state(state).await
        {
            let Some((_token, user)) =
                access_token::Entity::find_by_id(bearer_token.token().into())
                    .filter(access_token::Column::ExpiredAt.gt(Utc::now()))
                    .find_also_related(user::Entity)
                    .one(&state.db_conn)
                    .await
                    .map_err(Error::from)?
            else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };

            Ok(Self(user))
        } else {
            Ok(Self(None))
        }
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

        let content = if content_type == ContentType::form_url_encoded() {
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
impl FromRequest<State, Body> for SignedActivity {
    type Rejection = Response;

    async fn from_request(
        req: http::Request<Body>,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        let headers = req.headers().clone();
        let method = req.method().clone();
        let uri = req.uri().clone();

        let Json(activity): Json<Activity> =
            req.extract().await.map_err(IntoResponse::into_response)?;

        let ap_id = activity
            .rest
            .attributed_to()
            .ok_or_else(|| StatusCode::BAD_REQUEST.into_response())?;

        let remote_user = state.fetcher.fetch_actor(ap_id).await?;
        let Some(public_key) = remote_user.public_key()? else {
            error!(user_id = %remote_user.id, "Missing RSA public key");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        };

        let is_valid = crate::blocking::cpu(move || {
            let request = Request {
                headers: &headers,
                method: &method,
                uri: &uri,
            };

            phenomenon_http_signatures::verify(request, public_key.to_pkcs1_der()?.as_bytes())
                .map_err(Error::from)
        })
        .await??;

        if !is_valid {
            return Err(StatusCode::UNAUTHORIZED.into_response());
        }

        Ok(Self(activity))
    }
}
