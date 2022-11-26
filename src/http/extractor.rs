use crate::{
    db::entity::{oauth::access_token, user},
    error::Error,
    state::State,
};
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Form, Json, TypedHeader,
};
use chrono::Utc;
use headers::{authorization::Bearer, Authorization};
use phenomenon_http_signatures::Request;
use phenomenon_model::ap::Activity;
use rsa::pkcs1::EncodeRsaPublicKey;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::de::DeserializeOwned;

pub struct AuthExtactor(pub Option<user::Model>);

#[async_trait]
impl FromRequest<Body> for AuthExtactor {
    type Rejection = Response;

    async fn from_request(req: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        let Extension(state) = Extension::<State>::from_request(req)
            .await
            .map_err(IntoResponse::into_response)?;

        if let Ok(TypedHeader(Authorization(bearer_token))) =
            TypedHeader::<Authorization<Bearer>>::from_request(req).await
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
impl<T> FromRequest<Body> for FormOrJson<T>
where
    T: DeserializeOwned + Send,
{
    type Rejection = Response;

    async fn from_request(req: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        if let Ok(Form(data)) = Form::from_request(req).await {
            Ok(Self(data))
        } else {
            let Json(data) = Json::from_request(req)
                .await
                .map_err(IntoResponse::into_response)?;

            Ok(Self(data))
        }
    }
}

pub struct SignedActivity(pub Activity);

#[async_trait]
impl FromRequest<Body> for SignedActivity {
    type Rejection = Response;

    async fn from_request(parts: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        let (Extension(state), Json(activity)) =
            <(Extension<State>, Json<Activity>)>::from_request(parts)
                .await
                .map_err(IntoResponse::into_response)?;

        let Some(ap_id) = activity.rest.attributed_to() else {
            return Err(StatusCode::BAD_REQUEST.into_response());
        };
        let remote_user = state.fetcher.fetch_actor(ap_id).await?;
        let Some(public_key) = remote_user.public_key()? else {
            error!(user_id = %remote_user.id, "Missing RSA public key");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        };

        let headers = parts.headers().clone();
        let method = parts.method().clone();
        let uri = parts.uri().clone();
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
