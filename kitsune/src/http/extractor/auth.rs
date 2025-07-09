use crate::state::Zustand;
use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, OptionalFromRequestParts},
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use headers::{Authorization, authorization::Bearer};
use http::request::Parts;
use kitsune_db::{
    model::{account::Account, user::User},
    schema::{accounts, oauth2_access_tokens, users},
    with_connection,
};
use kitsune_error::{Error, Result};
use time::OffsetDateTime;
use trials::attempt;

/// Mastodon-specific auth extractor alias
///
/// Mastodon won't let access token expire ever. I don't know why, but they just don't.
/// Instead of hacking some special case for the Mastodon API into our database schema, we just don't enforce token expiration.
#[cfg(feature = "mastodon-api")]
pub type MastodonAuthExtractor = AuthExtractor<false>;

#[derive(Clone)]
pub struct UserData {
    pub account: Account,
    pub user: User,
}

/// Extract the account and user from the request
///
/// The const generics parameter `ENFORCE_EXPIRATION` lets you toggle whether the extractor should ignore the expiration date.
/// This is needed for compatibility with the Mastodon API, more information in the docs of the [`MastodonAuthExtractor`] type alias.
pub struct AuthExtractor<const ENFORCE_EXPIRATION: bool = true>(pub UserData);

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

        let mut user_account_query = oauth2_access_tokens::table
            .inner_join(users::table)
            .inner_join(accounts::table.on(accounts::id.eq(users::account_id)))
            .filter(oauth2_access_tokens::token.eq(bearer_token.token()))
            .into_boxed();

        if ENFORCE_EXPIRATION {
            user_account_query = user_account_query
                .filter(oauth2_access_tokens::expires_at.gt(OffsetDateTime::now_utc()));
        }

        let result: Result<(User, Account)> = attempt! { async
            with_connection!(state.db_pool, |db_conn| {
                user_account_query
                    .select(<(User, Account)>::as_select())
                    .get_result(db_conn)
                    .await
                    .map_err(Error::from)
            })?
        };

        let (user, account) = result?;

        Ok(Self(UserData { account, user }))
    }
}

impl<const ENFORCE_EXPIRATION: bool> OptionalFromRequestParts<Zustand>
    for AuthExtractor<ENFORCE_EXPIRATION>
{
    type Rejection = Response;

    #[inline]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &Zustand,
    ) -> Result<Option<Self>, Self::Rejection> {
        // just silently swallow any auth errors
        let value = <Self as FromRequestParts<_>>::from_request_parts(parts, state)
            .await
            .ok();

        Ok(value)
    }
}
