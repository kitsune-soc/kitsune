use crate::{
    activitypub::Fetcher,
    cache::Cache,
    consts::MENTION_REGEX,
    db::model::{
        account, mention,
        post::{self, Visibility},
    },
    error::Error as ServerError,
    http::graphql::ContextExt,
    sanitize::CleanHtmlExt,
    webfinger::Webfinger,
};
use async_graphql::{Context, Error, Object, Result};
use chrono::Utc;
use fancy_regex::{Captures, Match};
use futures_util::FutureExt;
use pulldown_cmark::{html, Options, Parser};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter, TransactionTrait,
};
use tokio::runtime::Handle;
use uuid::Uuid;

#[derive(Clone)]
pub struct MentionResolver<FPC, FUC, WC> {
    db_conn: DatabaseConnection,
    fetcher: Fetcher<FPC, FUC>,
    webfinger: Webfinger<WC>,
}

impl<FPC, FUC, WC> MentionResolver<FPC, FUC, WC>
where
    FPC: Cache<str, post::Model> + Clone + Send + Sync + 'static,
    FUC: Cache<str, account::Model> + Clone + Send + Sync + 'static,
    WC: Cache<str, String> + Clone + Send + Sync + 'static,
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

    pub async fn resolve(&self, content: String) -> Result<(Vec<Uuid>, String)> {
        let this = self.clone();
        let handle = Handle::current();

        // TODO: Find a better way of parsing mentions. I hate this.
        tokio::task::spawn_blocking(move || {
            let mut mentioned_account_ids = Vec::new();
            let content = MENTION_REGEX
                .replace_all(&content, |capture: &Captures<'_>| {
                    let username = &capture[1];
                    let domain = capture.get(2).as_ref().map(Match::as_str);
                    let Some(account) = handle.block_on(this.fetch_account(username, domain)).ok().flatten() else {
                        return capture[0].to_string();
                    };

                    mentioned_account_ids.push(account.id);
                    format!("<a class=\"mention\" href=\"{}\">{}</a>", account.url, &capture[0])
                })
                .to_string();

            (mentioned_account_ids, content)
        })
        .await
        .map_err(Error::from)
    }
}

#[derive(Default)]
pub struct PostMutation;

#[Object]
impl PostMutation {
    pub async fn create_post(
        &self,
        ctx: &Context<'_>,
        content: String,
        is_sensitive: bool,
        visibility: Visibility,
    ) -> Result<post::Model> {
        let state = ctx.state();
        let user = ctx.user_data()?;
        let content = {
            let parser = Parser::new_ext(&content, Options::all());
            let mut buf = String::new();
            html::push_html(&mut buf, parser);
            buf.clean_html();
            buf
        };

        // TODO: Cache this resolver somewhere
        let mention_resolver = MentionResolver::new(
            state.db_conn.clone(),
            state.fetcher.clone(),
            state.webfinger.clone(),
        );
        let (mentioned_account_ids, content) = mention_resolver.resolve(content).await?;

        let id = Uuid::new_v4();
        let account_id = user.account.id;
        let url = format!("https://{}/posts/{id}", state.config.domain);

        state
            .db_conn
            .transaction(move |tx| {
                async move {
                    let post = post::Model {
                        id,
                        account_id,
                        subject: None,
                        content,
                        is_sensitive,
                        visibility,
                        url,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }
                    .into_active_model()
                    .insert(tx)
                    .await?;

                    for account_id in mentioned_account_ids {
                        mention::Model {
                            account_id,
                            post_id: post.id,
                        }
                        .into_active_model()
                        .insert(tx)
                        .await?;
                    }

                    Ok::<_, ServerError>(post)
                }
                .boxed()
            })
            .await
            .map_err(Error::from)
    }

    pub async fn delete_post(&self, ctx: &Context<'_>, id: Uuid) -> Result<Uuid> {
        let state = ctx.state();
        let user = ctx.user_data()?;

        let post = post::Entity::find_by_id(id)
            .belongs_to(&user.account)
            .one(&state.db_conn)
            .await?
            .ok_or_else(|| Error::new("Post not found"))?;

        // TODO: Send out delete activity

        post.delete(&state.db_conn).await?;

        Ok(id)
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
