use crate::{
    db::entity::{
        oauth::{access_token, application},
        user,
    },
    error::Error,
    state::State,
};
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequest, Query, RequestParts},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, TypedHeader,
};
use chrono::Utc;
use headers::{
    authorization::{Basic, Bearer},
    Authorization,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use std::str::FromStr;
use uuid::Uuid;

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

#[derive(Deserialize)]
struct OAuthApplicationQuery {
    client_id: Uuid,
    client_secret: String,
}

pub struct OAuthApplication(pub application::Model);

#[async_trait]
impl FromRequest<Body> for OAuthApplication {
    type Rejection = Response;

    async fn from_request(req: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        let Extension(state) = Extension::<State>::from_request(req)
            .await
            .map_err(IntoResponse::into_response)?;

        let (client_id, client_secret) = if let Ok(TypedHeader(Authorization(creds))) =
            TypedHeader::<Authorization<Basic>>::from_request(req).await
        {
            (
                Uuid::from_str(creds.username()).map_err(Error::from)?,
                creds.password().to_string(),
            )
        } else {
            let Query(query) = Query::<OAuthApplicationQuery>::from_request(req)
                .await
                .map_err(IntoResponse::into_response)?;
            (query.client_id, query.client_secret)
        };

        let application = application::Entity::find_by_id(client_id)
            .filter(application::Column::Secret.eq(client_secret))
            .one(&state.db_conn)
            .await
            .map_err(Error::from)?
            .ok_or(Error::OAuthApplicationNotFound)?;

        Ok(Self(application))
    }
}
