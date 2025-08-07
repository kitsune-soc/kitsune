use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{model::Oauth2Application, schema::oauth2_applications, with_connection};
use komainu::{ClientExtractor, scope::Scope};
use speedy_uuid::Uuid;
use std::{borrow::Cow, str::FromStr};
use trials::attempt;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Extractor {
    db_pool: kitsune_db::PgPool,
}

impl ClientExtractor for Extractor {
    #[cfg_attr(not(coverage), instrument(skip_all, fields(client_id)))]
    async fn extract(
        &self,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Result<komainu::Client<'_>, komainu::error::Error> {
        let client_id = Uuid::from_str(client_id).unwrap();

        let result: Result<_, kitsune_error::Error> = attempt! { async
            with_connection!(self.db_pool, |db_conn| {
                let mut query = oauth2_applications::table.find(client_id).into_boxed();

                if let Some(client_secret) = client_secret {
                    query = query.filter(oauth2_applications::secret.eq(client_secret));
                }

                query.first::<Oauth2Application>(db_conn).await
            })?
        };

        // ToDo: error handling
        let client = result.unwrap();

        Ok(komainu::Client {
            client_id: Cow::Owned(client.id.to_string()),
            client_secret: Cow::Owned(client.secret),
            scopes: Scope::from_str(&client.scopes).unwrap(),
            redirect_uri: Cow::Owned(client.redirect_uri),
        })
    }
}
