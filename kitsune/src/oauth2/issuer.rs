use super::{chrono_to_timestamp, timestamp_to_chrono};
use async_trait::async_trait;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    catch_error,
    model::oauth2,
    schema::{oauth2_access_tokens, oauth2_applications, oauth2_refresh_tokens},
    with_connection, with_transaction, PgPool,
};
use kitsune_error::Error;
use kitsune_util::generate_secret;
use oxide_auth::primitives::{
    grant::{Extensions, Grant},
    issuer::{RefreshedToken, TokenType},
    prelude::IssuedToken,
};
use oxide_auth_async::primitives::Issuer;

#[derive(Clone)]
pub struct OAuthIssuer {
    pub db_pool: PgPool,
}

#[async_trait]
impl Issuer for OAuthIssuer {
    async fn issue(&mut self, grant: Grant) -> Result<IssuedToken, ()> {
        let application_id = grant.client_id.parse().map_err(|_| ())?;
        let user_id = grant.owner_id.parse().map_err(|_| ())?;
        let scopes = grant.scope.to_string();
        let expires_at = chrono_to_timestamp(grant.until);

        let (access_token, refresh_token) = catch_error!(with_transaction!(self.db_pool, |tx| {
            let access_token = diesel::insert_into(oauth2_access_tokens::table)
                .values(oauth2::NewAccessToken {
                    token: generate_secret().as_str(),
                    user_id: Some(user_id),
                    application_id: Some(application_id),
                    scopes: scopes.as_str(),
                    expires_at,
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

            Ok::<_, Error>((access_token, refresh_token))
        }))
        .map_err(|_| ())?
        .map_err(|_| ())?;

        Ok(IssuedToken {
            token: access_token.token,
            refresh: Some(refresh_token.token),
            until: grant.until,
            token_type: TokenType::Bearer,
        })
    }

    async fn refresh(&mut self, refresh_token: &str, grant: Grant) -> Result<RefreshedToken, ()> {
        let (refresh_token, access_token) =
            catch_error!(with_connection!(self.db_pool, |db_conn| {
                oauth2_refresh_tokens::table
                    .find(refresh_token)
                    .inner_join(oauth2_access_tokens::table)
                    .select(<(oauth2::RefreshToken, oauth2::AccessToken)>::as_select())
                    .get_result::<(oauth2::RefreshToken, oauth2::AccessToken)>(db_conn)
                    .await
            }))
            .map_err(|_| ())?
            .map_err(|_| ())?;

        let (access_token, refresh_token) = catch_error!(with_transaction!(self.db_pool, |tx| {
            let new_access_token = diesel::insert_into(oauth2_access_tokens::table)
                .values(oauth2::NewAccessToken {
                    user_id: access_token.user_id,
                    token: generate_secret().as_str(),
                    application_id: access_token.application_id,
                    scopes: access_token.scopes.as_str(),
                    expires_at: chrono_to_timestamp(grant.until),
                })
                .get_result::<oauth2::AccessToken>(tx)
                .await?;

            let refresh_token = diesel::update(&refresh_token)
                .set(oauth2_refresh_tokens::access_token.eq(new_access_token.token.as_str()))
                .get_result::<oauth2::RefreshToken>(tx)
                .await?;

            diesel::delete(&access_token).execute(tx).await?;

            Ok::<_, Error>((new_access_token, refresh_token))
        }))
        .map_err(|_| ())?
        .map_err(|_| ())?;

        Ok(RefreshedToken {
            token: access_token.token,
            refresh: Some(refresh_token.token),
            until: timestamp_to_chrono(access_token.expires_at),
            token_type: TokenType::Bearer,
        })
    }

    async fn recover_token(&mut self, access_token: &str) -> Result<Option<Grant>, ()> {
        let oauth_data = catch_error!(with_connection!(self.db_pool, |db_conn| {
            oauth2_access_tokens::table
                .find(access_token)
                .inner_join(oauth2_applications::table)
                .select(<(oauth2::AccessToken, oauth2::Application)>::as_select())
                .get_result::<(oauth2::AccessToken, oauth2::Application)>(db_conn)
                .await
                .optional()
        }))
        .map_err(|_| ())?
        .map_err(|_| ())?;

        let oauth_data = oauth_data.map(|(access_token, app)| {
            let scope = app.scopes.parse().unwrap();
            let redirect_uri = app.redirect_uri.parse().unwrap();
            let until = timestamp_to_chrono(access_token.expires_at);

            Grant {
                owner_id: access_token
                    .user_id
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                client_id: app.id.to_string(),
                scope,
                redirect_uri,
                until,
                extensions: Extensions::default(),
            }
        });

        Ok(oauth_data)
    }

    async fn recover_refresh(&mut self, refresh_token: &str) -> Result<Option<Grant>, ()> {
        let oauth_data = catch_error!(with_connection!(self.db_pool, |db_conn| {
            oauth2_refresh_tokens::table
                .find(refresh_token)
                .inner_join(oauth2_access_tokens::table)
                .inner_join(oauth2_applications::table)
                .select(<(oauth2::AccessToken, oauth2::Application)>::as_select())
                .get_result::<(oauth2::AccessToken, oauth2::Application)>(db_conn)
                .await
                .optional()
        }))
        .map_err(|_| ())?
        .map_err(|_| ())?;

        let oauth_data = oauth_data.map(|(access_token, app)| {
            let scope = access_token.scopes.parse().unwrap();
            let redirect_uri = app.redirect_uri.parse().unwrap();
            let until = chrono::NaiveDateTime::MAX.and_utc();

            Grant {
                owner_id: access_token
                    .user_id
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                client_id: app.id.to_string(),
                scope,
                redirect_uri,
                until,
                extensions: Extensions::default(),
            }
        });

        Ok(oauth_data)
    }
}
