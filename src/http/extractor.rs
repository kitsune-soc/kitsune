use crate::{
    db::entity::{token, user},
    error::Error,
    state::State,
};
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, TypedHeader,
};
use headers::{authorization::Bearer, Authorization};
use sea_orm::EntityTrait;

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
                token::Entity::find_by_id(bearer_token.token().into())
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
