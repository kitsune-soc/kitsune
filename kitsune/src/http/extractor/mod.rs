use crate::error::Result;
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::FromRequest,
    response::{IntoResponse, Response},
    Form, Json, RequestExt, TypedHeader,
};
use headers::ContentType;
use mime::Mime;
use serde::de::DeserializeOwned;

pub use self::{
    auth::{AuthExtractor, MastodonAuthExtractor, UserData},
    signed_activity::SignedActivity,
};

mod auth;
mod signed_activity;

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
