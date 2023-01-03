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
use kitsune_http_signatures::Request;
use kitsune_type::ap::Activity;
use rsa::pkcs8::Document;
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
impl FromRequest<Zustand, Body> for SignedActivity {
    type Rejection = Response;

    async fn from_request(
        req: http::Request<Body>,
        state: &Zustand,
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
        let (_tag, public_key) =
            Document::from_pem(&remote_user.public_key).map_err(Error::from)?;

        let is_valid = crate::blocking::cpu(move || {
            let request = Request {
                headers: &headers,
                method: &method,
                uri: &uri,
            };

            kitsune_http_signatures::verify(request, public_key.as_bytes()).map_err(Error::from)
        })
        .await??;

        if !is_valid {
            return Err(StatusCode::UNAUTHORIZED.into_response());
        }

        Ok(Self(activity))
    }
}
