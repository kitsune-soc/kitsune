use async_trait::async_trait;
use axum::{
    body::Body,
    extract::FromRequest,
    response::{IntoResponse, Response},
    Form, RequestExt,
};
use axum_extra::TypedHeader;
use headers::ContentType;
use http::StatusCode;
use kitsune_error::Result;
use mime::Mime;
use serde::de::DeserializeOwned;

pub use self::{
    auth::{AuthExtractor, UserData},
    json::Json,
    signed_activity::SignedActivity,
};

#[cfg(feature = "mastodon-api")]
pub use self::auth::MastodonAuthExtractor;

mod auth;
mod json;
mod signed_activity;

pub struct AgnosticForm<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for AgnosticForm<T>
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
        let content_type = Mime::from(content_type);

        let content = if content_type.essence_str() == mime::APPLICATION_WWW_FORM_URLENCODED {
            Form::from_request(req, state)
                .await
                .map_err(IntoResponse::into_response)?
                .0
        } else if content_type.essence_str() == mime::APPLICATION_JSON {
            Json::from_request(req, state)
                .await
                .map_err(IntoResponse::into_response)?
                .0
        } else {
            error!(%content_type, "Unknown content type");
            return Err(StatusCode::UNPROCESSABLE_ENTITY.into_response());
        };

        Ok(Self(content))
    }
}
