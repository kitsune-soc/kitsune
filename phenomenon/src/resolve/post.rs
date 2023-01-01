use crate::{
    activitypub::Fetcher,
    cache::Cache,
    db::model::{account, post},
    error::{Error, Result},
    webfinger::Webfinger,
};
use futures_util::FutureExt;
use parking_lot::Mutex;
use post_process::{BoxError, Element, Html, Transformer};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::{borrow::Cow, collections::HashSet, mem, sync::Arc};
use uuid::Uuid;

pub struct PostResolver<FPC, FUC, WC> {
    db_conn: DatabaseConnection,
    fetcher: Fetcher<FPC, FUC>,
    webfinger: Webfinger<WC>,
}

impl<FPC, FUC, WC> PostResolver<FPC, FUC, WC>
where
    FPC: Cache<str, post::Model> + Send + Sync,
    FUC: Cache<str, account::Model> + Send + Sync,
    WC: Cache<str, String> + Send + Sync,
{
    pub fn new(
        db_conn: DatabaseConnection,
        fetcher: Fetcher<FPC, FUC>,
        webfinger: Webfinger<WC>,
    ) -> Self {
        Self {
            db_conn,
            fetcher,
            webfinger,
        }
    }

    async fn fetch_account(
        &self,
        username: &str,
        domain: Option<&str>,
    ) -> Result<Option<account::Model>> {
        if let Some(domain) = domain {
            let Some(actor_url) = self.webfinger.fetch_actor_url(username, domain).await? else {
                return Ok(None)
            };

            self.fetcher
                .fetch_actor(&actor_url)
                .await
                .map(Some)
                .map_err(Error::from)
        } else {
            account::Entity::find()
                .filter(
                    account::Column::Username
                        .eq(username)
                        .and(account::Column::Domain.is_null()),
                )
                .one(&self.db_conn)
                .await
                .map_err(Error::from)
        }
    }

    async fn transform<'a>(
        &'a self,
        element: Element<'a>,
        mentioned_accounts: Arc<Mutex<HashSet<Uuid>>>,
    ) -> Result<Element<'a>, BoxError> {
        let element = match element {
            Element::Mention(mention) => {
                if let Some(account) = self
                    .fetch_account(&mention.username, mention.domain.as_deref())
                    .await?
                {
                    mentioned_accounts.lock().insert(account.id);

                    Element::Html(Html {
                        tag: Cow::Borrowed("a"),
                        attributes: vec![(Cow::Borrowed("href"), Cow::Owned(account.url))],
                        content: Box::new(Element::Mention(mention)),
                    })
                } else {
                    Element::Mention(mention)
                }
            }
            Element::Link(link) => Element::Html(Html {
                tag: Cow::Borrowed("a"),
                attributes: vec![(Cow::Borrowed("href"), link.content.clone())],
                content: Box::new(Element::Link(link)),
            }),
            elem => elem,
        };

        Ok(element)
    }

    /// Resolve the mentions inside a post
    ///
    /// # Returns
    ///
    /// - List of mentioned accounts
    /// - Content with the mentions replaced by links
    ///
    /// # Panics
    ///
    /// This should never panic
    pub async fn resolve(&self, content: &str) -> Result<(Vec<Uuid>, String)> {
        let mentioned_account_ids = Arc::new(Mutex::new(HashSet::new()));
        let transformer = Transformer::new(|elem| {
            let mentioned_account_ids = Arc::clone(&mentioned_account_ids);
            self.transform(elem, mentioned_account_ids).boxed()
        });

        let content = transformer
            .transform(content)
            .await
            .map_err(Error::PostProcessing)?;

        let mentioned_account_ids = mem::take(&mut *mentioned_account_ids.lock());
        Ok((mentioned_account_ids.into_iter().collect(), content))
    }
}

#[cfg(test)]
mod test {
    use super::PostResolver;
    use crate::{activitypub::Fetcher, cache::NoopCache, db::model::account, webfinger::Webfinger};
    use migration::{Migrator, MigratorTrait};
    use sea_orm::{Database, DatabaseConnection, EntityTrait};

    async fn db_conn() -> DatabaseConnection {
        let db_conn = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory SQLite");
        Migrator::up(&db_conn, None)
            .await
            .expect("Failed to migrate database");

        db_conn
    }

    #[tokio::test]
    async fn parse_mentions() {
        let db_conn = db_conn().await;
        let post = "Hello @0x0@corteximplant.com! How are you doing?";

        let mention_resolver = PostResolver::new(
            db_conn.clone(),
            Fetcher::new(db_conn.clone(), NoopCache, NoopCache),
            Webfinger::new(NoopCache),
        );
        let (mentioned_account_ids, content) = mention_resolver
            .resolve(post)
            .await
            .expect("Failed to resolve mentions");

        assert_eq!(content, "Hello <a href=\"https://corteximplant.com/users/0x0\">@0x0@corteximplant.com</a>! How are you doing?");
        assert_eq!(mentioned_account_ids.len(), 1);

        let mentioned_account = account::Entity::find_by_id(mentioned_account_ids[0])
            .one(&db_conn)
            .await
            .ok()
            .flatten()
            .expect("Failed to fetch account");

        assert_eq!(mentioned_account.username, "0x0");
        assert_eq!(mentioned_account.domain, Some("corteximplant.com".into()));
        assert_eq!(mentioned_account.url, "https://corteximplant.com/users/0x0");
    }
}
