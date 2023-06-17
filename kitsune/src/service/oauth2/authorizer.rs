use crate::util::generate_secret;
use async_trait::async_trait;
use diesel::{OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::oauth2,
    schema::{oauth2_applications, oauth2_authorization_codes},
    PgPool,
};
use oxide_auth::primitives::grant::{Extensions, Grant};
use oxide_auth_async::primitives::Authorizer;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct OAuthAuthorizer {
    pub db_pool: PgPool,
}

#[async_trait]
impl Authorizer for OAuthAuthorizer {
    async fn authorize(&mut self, grant: Grant) -> Result<String, ()> {
        let application_id = grant.client_id.parse().map_err(|_| ())?;
        let user_id = grant.owner_id.parse().map_err(|_| ())?;
        let scopes = grant.scope.to_string();
        let expired_at = OffsetDateTime::from_unix_timestamp(grant.until.timestamp())
            .unwrap()
            .replace_nanosecond(grant.until.timestamp_subsec_nanos())
            .unwrap();

        let mut db_conn = self.db_pool.get().await.map_err(|_| ())?;
        diesel::insert_into(oauth2_authorization_codes::table)
            .values(oauth2::NewAuthorizationCode {
                code: generate_secret().as_str(),
                application_id,
                user_id,
                scopes: scopes.as_str(),
                expired_at,
            })
            .returning(oauth2_authorization_codes::code)
            .get_result(&mut db_conn)
            .await
            .map_err(|_| ())
    }

    async fn extract(&mut self, authorization_code: &str) -> Result<Option<Grant>, ()> {
        let mut conn = self.db_pool.get().await.map_err(|_| ())?;
        let oauth_data = oauth2_authorization_codes::table
            .find(authorization_code)
            .inner_join(oauth2_applications::table)
            .first::<(oauth2::AuthorizationCode, oauth2::Application)>(&mut conn)
            .await
            .optional()
            .map_err(|_| ())?;

        let oauth_data = oauth_data.map(|(code, app)| {
            let scope = app.scopes.parse().unwrap();
            let redirect_uri = app.redirect_uri.parse().unwrap();
            let until = chrono::NaiveDateTime::from_timestamp_opt(
                code.expired_at.unix_timestamp(),
                code.expired_at.nanosecond(),
            )
            .unwrap()
            .and_utc();

            Grant {
                owner_id: code.user_id.to_string(),
                client_id: code.application_id.to_string(),
                scope,
                redirect_uri,
                until,
                extensions: Extensions::default(),
            }
        });

        Ok(oauth_data)
    }
}
