use crate::{
    consts::MENTION_REGEX,
    db::model::{
        account, mention,
        post::{self, Visibility},
    },
    error::Error as ServerError,
    http::graphql::ContextExt,
    sanitize::CleanHtmlExt,
    state::Zustand,
};
use async_graphql::{Context, Error, Object, Result};
use chrono::Utc;
use fancy_regex::{Captures, Match};
use futures_util::FutureExt;
use pulldown_cmark::{html, Options, Parser};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter,
    TransactionTrait,
};
use tokio::runtime::Handle;
use uuid::Uuid;

async fn fetch_account(
    state: &Zustand,
    username: &str,
    domain: Option<&str>,
) -> Result<Option<account::Model>> {
    if let Some(domain) = domain {
        let Some(actor_url) = state.webfinger.fetch_actor_url(username, domain).await? else {
            return Ok(None)
        };

        state
            .fetcher
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
            .one(&state.db_conn)
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

        let handle = Handle::current();
        let (mentioned_account_ids, content) = {
            let state = state.clone();

            tokio::task::spawn_blocking(move || {
                let mut mentioned_account_ids = Vec::new();
                let content = MENTION_REGEX
                    .replace_all(&content, |capture: &Captures<'_>| {
                        let username = &capture[1];
                        let domain = capture.get(2).as_ref().map(Match::as_str);
                        let Some(account) = handle.block_on(fetch_account(&state, username, domain)).ok().flatten() else {
                            return capture[0].to_string();
                        };

                        mentioned_account_ids.push(account.id);
                        format!("<a href=\"{}\">{}</a>", account.url, &capture[0])
                    })
                    .to_string();

                (mentioned_account_ids, content)
            })
            .await?
        };

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
