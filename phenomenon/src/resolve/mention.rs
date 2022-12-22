use crate::{
    activitypub::Fetcher,
    cache::Cache,
    consts::MENTION_REGEX,
    db::model::{account, post},
    error::{Error, Result},
    webfinger::Webfinger,
};
use fancy_regex::Match;
use itertools::Itertools;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

struct MentionData<'a> {
    full_mention: &'a str,
    username: &'a str,
    domain: Option<&'a str>,
}

pub struct MentionResolver<FPC, FUC, WC> {
    db_conn: DatabaseConnection,
    fetcher: Fetcher<FPC, FUC>,
    webfinger: Webfinger<WC>,
}

impl<FPC, FUC, WC> MentionResolver<FPC, FUC, WC>
where
    FPC: Cache<str, post::Model>,
    FUC: Cache<str, account::Model>,
    WC: Cache<str, String>,
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
    pub async fn resolve(&self, content: String) -> Result<(Vec<Uuid>, String)> {
        let mention_data = MENTION_REGEX
            .captures_iter(&content)
            .filter_map(|capture| {
                let capture = capture.ok()?;

                // Call to `.get` needed to satisfy lifetime requirements
                let full_mention = capture.get(0).unwrap().as_str();
                let username = capture.get(1).unwrap().as_str();
                let domain = capture.get(2).as_ref().map(Match::as_str);

                Some(MentionData {
                    full_mention,
                    username,
                    domain,
                })
            })
            .unique_by(|mention_data| mention_data.full_mention);

        let mut content = content.clone();
        let mut mentioned_account_ids = Vec::new();
        for mention in mention_data {
            if let Some(account) = self
                .fetch_account(mention.username, mention.domain)
                .await
                .ok()
                .flatten()
            {
                mentioned_account_ids.push(account.id);

                let formatted_link =
                    format!("<a href=\"{}\">{}</a>", account.url, mention.full_mention);
                content = content.replace(mention.full_mention, &formatted_link);
            }
        }

        Ok((mentioned_account_ids, content))
    }
}

#[cfg(test)]
mod test {
    use super::MentionResolver;
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

        let mention_resolver = MentionResolver::new(
            db_conn.clone(),
            Fetcher::new(db_conn.clone(), NoopCache, NoopCache),
            Webfinger::new(NoopCache),
        );
        let (mentioned_account_ids, content) = mention_resolver
            .resolve(post.to_string())
            .await
            .expect("Failed to resolve mentions");

        assert_eq!(content, "Hello <a class=\"mention\" href=\"https://corteximplant.com/users/0x0\">@0x0@corteximplant.com</a>! How are you doing?");
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
