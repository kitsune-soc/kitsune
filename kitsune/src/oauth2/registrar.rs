use async_trait::async_trait;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{model::oauth2, schema::oauth2_applications, PgPool};
use oxide_auth::{
    endpoint::{PreGrant, Scope},
    primitives::registrar::{BoundClient, ClientUrl, ExactUrl, RegisteredUrl, RegistrarError},
};
use oxide_auth_async::primitives::Registrar;
use scoped_futures::ScopedFutureExt;
use speedy_uuid::Uuid;
use std::{
    borrow::Cow,
    str::{self, FromStr},
};

use super::OAuthScope;

#[derive(Clone)]
pub struct OAuthRegistrar {
    pub db_pool: PgPool,
}

#[async_trait]
impl Registrar for OAuthRegistrar {
    async fn bound_redirect<'a>(
        &self,
        bound: ClientUrl<'a>,
    ) -> Result<BoundClient<'a>, RegistrarError> {
        if let Some(redirect_uri) = bound.redirect_uri {
            Ok(BoundClient {
                client_id: bound.client_id,
                redirect_uri: Cow::Owned(RegisteredUrl::Exact(redirect_uri.into_owned())),
            })
        } else {
            Err(RegistrarError::Unspecified)
        }
    }

    async fn negotiate<'a>(
        &self,
        client: BoundClient<'a>,
        scope: Option<Scope>,
    ) -> Result<PreGrant, RegistrarError> {
        let client_id: Uuid = client
            .client_id
            .parse()
            .map_err(|_| RegistrarError::PrimitiveError)?;

        let client = self
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    oauth2_applications::table
                        .find(client_id)
                        .filter(oauth2_applications::redirect_uri.eq(client.redirect_uri.as_str()))
                        .get_result::<oauth2::Application>(db_conn)
                        .await
                        .optional()
                }
                .scoped()
            })
            .await
            .map_err(|_| RegistrarError::PrimitiveError)?
            .ok_or(RegistrarError::Unspecified)?;

        let client_id = client.id.to_string();
        let redirect_uri = ExactUrl::new(client.redirect_uri)
            .map_err(|_| RegistrarError::PrimitiveError)?
            .into();

        let scope = if let Some(scope) = scope {
            let mut valid_scopes = scope
                .iter()
                .filter(|scope| OAuthScope::from_str(scope).is_ok())
                .peekable();

            // Default to read scope if no scopes are defined
            if valid_scopes.peek().is_none() {
                OAuthScope::Read.as_ref().parse().unwrap()
            } else {
                itertools::intersperse(valid_scopes, " ")
                    .collect::<String>()
                    .parse()
                    .unwrap()
            }
        } else {
            OAuthScope::Read.as_ref().parse().unwrap()
        };

        Ok(PreGrant {
            client_id,
            redirect_uri,
            scope,
        })
    }

    async fn check(
        &self,
        client_id: &str,
        passphrase: Option<&[u8]>,
    ) -> Result<(), RegistrarError> {
        let client_id: Uuid = client_id
            .parse()
            .map_err(|_| RegistrarError::PrimitiveError)?;
        let mut client_query = oauth2_applications::table.find(client_id).into_boxed();

        if let Some(passphrase) = passphrase {
            let passphrase = simdutf8::basic::from_utf8(passphrase)
                .map_err(|_| RegistrarError::PrimitiveError)?;
            client_query = client_query.filter(oauth2_applications::secret.eq(passphrase));
        }

        self.db_pool
            .with_connection(|db_conn| {
                async move {
                    client_query
                        .select(oauth2_applications::id)
                        .execute(db_conn)
                        .await
                        .optional()
                }
                .scoped()
            })
            .await
            .map_err(|_| RegistrarError::PrimitiveError)?
            .map(|_| ())
            .ok_or(RegistrarError::Unspecified)
    }
}
