use crate::oauth2::TOKEN_VALID_DURATION;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    insert::NewOauth2AccessToken,
    model::{Oauth2AccessToken, Oauth2RefreshToken},
    schema::{oauth2_access_tokens, oauth2_refresh_tokens},
    with_connection, with_transaction,
};
use kitsune_util::generate_secret;
use komainu::flow::{SuccessTokenResponse, TokenType, refresh};
use speedy_uuid::Uuid;
use std::{borrow::Cow, str::FromStr};
use trials::attempt;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Issuer {
    db_pool: kitsune_db::PgPool,
}

impl refresh::Issuer for Issuer {
    async fn issue_token(
        &self,
        client: &komainu::Client<'_>,
        refresh_token: &str,
    ) -> Result<komainu::flow::TokenResponse<'_>, komainu::flow::Error> {
        let client_id = Uuid::from_str(&client.client_id).unwrap();

        let result: Result<_, kitsune_error::Error> = attempt! { async
            let (refresh_token, access_token) =
                with_connection!(self.db_pool, |db_conn| {
                    oauth2_refresh_tokens::table
                        .find(refresh_token)
                        .filter(oauth2_refresh_tokens::application_id.eq(client_id))
                        .inner_join(oauth2_access_tokens::table)
                        .select(<(Oauth2RefreshToken, Oauth2AccessToken)>::as_select())
                        .get_result::<(Oauth2RefreshToken, Oauth2AccessToken)>(db_conn)
                        .await
                })?;

            with_transaction!(self.db_pool, |tx| {
                let new_access_token = diesel::insert_into(oauth2_access_tokens::table)
                    .values(NewOauth2AccessToken {
                        user_id: access_token.user_id,
                        token: generate_secret().as_str(),
                        application_id: access_token.application_id,
                        scopes: access_token.scopes.as_str(),
                        expires_at: Timestamp::now_utc() + TOKEN_VALID_DURATION,
                    })
                    .get_result::<Oauth2AccessToken>(tx)
                    .await?;

                let refresh_token = diesel::update(&refresh_token)
                    .set(oauth2_refresh_tokens::access_token.eq(new_access_token.token.as_str()))
                    .get_result::<Oauth2RefreshToken>(tx)
                    .await?;

                diesel::delete(&access_token).execute(tx).await?;

                Ok::<_, kitsune_error::Error>((new_access_token, refresh_token))
            })?
        };

        // ToDo: error handling
        let (access_token, refresh_token) = result.unwrap();

        Ok(SuccessTokenResponse {
            access_token: Cow::Owned(access_token.token),
            token_type: TokenType::Bearer,
            refresh_token: Cow::Owned(refresh_token.token),
            expires_in: access_token
                .expires_at
                .duration_since(Timestamp::now_utc())
                .whole_seconds() as _,
        }
        .into())
    }
}
