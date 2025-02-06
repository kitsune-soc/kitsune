use crate::oauth2::TOKEN_VALID_DURATION;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    model::oauth2,
    schema::{
        oauth2_access_tokens, oauth2_applications, oauth2_authorization_codes,
        oauth2_refresh_tokens,
    },
    with_connection, with_transaction,
};
use kitsune_util::generate_secret;
use komainu::{
    flow::{authorization, SuccessTokenResponse, TokenType},
    scope::Scope,
};
use speedy_uuid::Uuid;
use std::{borrow::Cow, str::FromStr};
use trials::attempt;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Issuer {
    db_pool: kitsune_db::PgPool,
}

impl authorization::Issuer for Issuer {
    #[instrument(skip_all)]
    async fn load_authorization(
        &self,
        auth_code: &str,
    ) -> Result<Option<komainu::Authorization<'_>>, komainu::flow::Error> {
        let result: Result<_, kitsune_error::Error> = attempt! { async
            with_connection!(self.db_pool, |db_conn| {
                oauth2_authorization_codes::table
                    .find(auth_code)
                    .inner_join(oauth2_applications::table)
                    .first::<(oauth2::AuthorizationCode, oauth2::Application)>(db_conn)
                    .await
                    .optional()
            })?
        };

        // ToDo: error handling
        let Some((code, application)) = result.unwrap() else {
            return Ok(None);
        };

        Ok(Some(komainu::Authorization {
            code: Cow::Owned(code.code),
            client: komainu::Client {
                client_id: Cow::Owned(application.id.to_string()),
                client_secret: Cow::Owned(application.secret),
                scopes: Scope::from_str(&application.scopes).unwrap(),
                redirect_uri: Cow::Owned(application.redirect_uri),
            },
            pkce_payload: None, // ToDo: store and load this
            scopes: Scope::from_str(&code.scopes).unwrap(),
            user_id: Cow::Owned(code.user_id.to_string()),
        }))
    }

    #[instrument(skip_all)]
    async fn issue_token(
        &self,
        authorization: &komainu::Authorization<'_>,
    ) -> Result<komainu::flow::TokenResponse<'_>, komainu::flow::Error> {
        let user_id = Uuid::from_str(&authorization.user_id).unwrap();
        let application_id = Uuid::from_str(&authorization.client.client_id).unwrap();
        let scopes = authorization.scopes.to_string();

        let result: Result<_, kitsune_error::Error> = attempt! { async
            with_transaction!(self.db_pool, |tx| {
                let access_token = diesel::insert_into(oauth2_access_tokens::table)
                    .values(oauth2::NewAccessToken {
                        user_id: Some(user_id),
                        application_id: Some(application_id),
                        token: generate_secret().as_str(),
                        scopes: &scopes,
                        expires_at: Timestamp::now_utc() + TOKEN_VALID_DURATION,
                    })
                    .returning(oauth2::AccessToken::as_returning())
                    .get_result::<oauth2::AccessToken>(tx)
                    .await?;

                let refresh_token = diesel::insert_into(oauth2_refresh_tokens::table)
                    .values(oauth2::NewRefreshToken {
                        token: generate_secret().as_str(),
                        access_token: access_token.token.as_str(),
                        application_id,
                    })
                    .returning(oauth2::RefreshToken::as_returning())
                    .get_result::<oauth2::RefreshToken>(tx)
                    .await?;

                diesel::delete(oauth2_authorization_codes::table.find(&authorization.code)).execute(tx).await?;

                Ok::<_, kitsune_error::Error>((access_token, refresh_token))
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
