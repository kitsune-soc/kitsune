use crate::{error::Error, state::Zustand};
use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    response::{IntoResponse, Response},
    RequestPartsExt, TypedHeader,
};
use headers::{authorization::Bearer, Authorization};
use http::{request::Parts, StatusCode};
use kitsune_db::entity::{
    accounts, oauth2_access_tokens,
    prelude::{Accounts, Oauth2AccessTokens, Users},
    users,
};
use sea_orm::{ColumnTrait, QueryFilter, Related};
use std::ops::{Deref, DerefMut};
use time::OffsetDateTime;

/// Mastodon-specific auth extractor alias
///
/// Mastodon won't let access token expire ever. I don't know why, but they just don't.
/// Instead of hacking some special case for the Mastodon API into our database schema, we just don't enforce token expiration.
#[cfg(feature = "mastodon-api")]
pub type MastodonAuthExtractor = AuthExtractor<false>;

#[derive(Clone)]
pub struct UserData {
    pub account: accounts::Model,
    pub user: users::Model,
}

/// Extract the account and user from the request
///
/// The const generics parameter `ENFORCE_EXPIRATION` lets you toggle whether the extractor should ignore the expiration date.
/// This is needed for compatibility with the Mastodon API, more information in the docs of the [`MastodonAuthExtractor`] type alias.
pub struct AuthExtractor<const ENFORCE_EXPIRATION: bool>(pub UserData);

impl<const ENFORCE_EXPIRATION: bool> Deref for AuthExtractor<ENFORCE_EXPIRATION> {
    type Target = UserData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const ENFORCE_EXPIRATION: bool> DerefMut for AuthExtractor<ENFORCE_EXPIRATION> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait]
impl<const ENFORCE_EXPIRATION: bool> FromRequestParts<Zustand>
    for AuthExtractor<ENFORCE_EXPIRATION>
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Zustand,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization::<Bearer>(bearer_token)) = parts
            .extract_with_state(state)
            .await
            .map_err(IntoResponse::into_response)?;

        let mut user_account_query = <Oauth2AccessTokens as Related<Users>>::find_related()
            .find_also_related(Accounts)
            .filter(oauth2_access_tokens::Column::Token.eq(bearer_token.token()));

        if ENFORCE_EXPIRATION {
            user_account_query = user_account_query
                .filter(oauth2_access_tokens::Column::ExpiredAt.gt(OffsetDateTime::now_utc()));
        }

        let Some((user, Some(account))) = user_account_query
            .one(&state.db_conn)
            .await
            .map_err(Error::from)?
        else {
            return Err(StatusCode::UNAUTHORIZED.into_response());
        };

        Ok(Self(UserData { account, user }))
    }
}
