use super::{chrono_to_timestamp, timestamp_to_chrono};
use async_trait::async_trait;
use diesel::{OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::oauth2,
    schema::{oauth2_applications, oauth2_authorization_codes},
    with_connection, PgPool,
};
use kitsune_util::generate_secret;
use oxide_auth::primitives::grant::{Extensions, Grant};
use oxide_auth_async::primitives::Authorizer;

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
        let secret = generate_secret();
        let expires_at = chrono_to_timestamp(grant.until);

        with_connection!(self.db_pool, |db_conn| {
            diesel::insert_into(oauth2_authorization_codes::table)
                .values(oauth2::NewAuthorizationCode {
                    code: secret.as_str(),
                    application_id,
                    user_id,
                    scopes: scopes.as_str(),
                    expires_at,
                })
                .returning(oauth2_authorization_codes::code)
                .get_result(db_conn)
                .await
        })
        .map_err(|_| ())
    }

    async fn extract(&mut self, authorization_code: &str) -> Result<Option<Grant>, ()> {
        let oauth_data = with_connection!(self.db_pool, |db_conn| {
            oauth2_authorization_codes::table
                .find(authorization_code)
                .inner_join(oauth2_applications::table)
                .first::<(oauth2::AuthorizationCode, oauth2::Application)>(db_conn)
                .await
                .optional()
        })
        .map_err(|_| ())?;

        let oauth_data = oauth_data.map(|(code, app)| {
            let scope = app.scopes.parse().unwrap();
            let redirect_uri = app.redirect_uri.parse().unwrap();

            Grant {
                owner_id: code.user_id.to_string(),
                client_id: code.application_id.to_string(),
                scope,
                redirect_uri,
                until: timestamp_to_chrono(code.expires_at),
                extensions: Extensions::default(),
            }
        });

        Ok(oauth_data)
    }
}
