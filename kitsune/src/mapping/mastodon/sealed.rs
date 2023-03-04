use crate::{
    error::{Error, Result},
    service::attachment::AttachmentService,
};
use async_trait::async_trait;
use futures_util::{future::OptionFuture, TryStreamExt};
use kitsune_db::entity::{
    accounts, posts, posts_mentions,
    prelude::{Accounts, Favourites, Posts, PostsMentions},
};
use kitsune_type::mastodon::{account::Source, status::Mention, Account, Status};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct MapperState<'a> {
    pub attachment_service: &'a AttachmentService,
    pub db_conn: &'a DatabaseConnection,
}

#[async_trait]
pub trait IntoMastodon {
    /// Mastodon API entity that gets returned
    type Output: Clone + DeserializeOwned + Serialize;

    /// Unique identifier of the object
    ///
    /// Returning the primary key of the database should be fine (our IDs are v7 UUIDs)
    fn id(&self) -> Option<Uuid>;

    /// Map something to its Mastodon API equivalent
    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output>;
}

#[async_trait]
impl IntoMastodon for accounts::Model {
    type Output = Account;

    fn id(&self) -> Option<Uuid> {
        Some(self.id)
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let statuses_count = Posts::find()
            .filter(posts::Column::AccountId.eq(self.id))
            .count(state.db_conn)
            .await?;
        let mut acct = self.username.clone();
        if let Some(domain) = self.domain {
            acct.push('@');
            acct.push_str(&domain);
        }

        let avatar = if let Some(avatar_id) = self.avatar_id {
            state.attachment_service.get_url(avatar_id).await?
        } else {
            "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into()
        };

        let header = if let Some(header_id) = self.header_id {
            state.attachment_service.get_url(header_id).await?
        } else {
            "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into()
        };

        Ok(Account {
            id: self.id,
            acct,
            username: self.username,
            display_name: self.display_name.unwrap_or_default(),
            created_at: self.created_at.into(),
            locked: self.locked,
            note: self.note.unwrap_or_default(),
            url: self.url,
            avatar_static: avatar.clone(),
            avatar,
            header_static: header.clone(),
            header,
            followers_count: 0,
            following_count: 0,
            statuses_count,
            source: Source {
                privacy: "public".into(),
                sensitive: false,
                language: String::new(),
                note: String::new(),
                fields: Vec::new(),
            },
        })
    }
}

#[async_trait]
impl IntoMastodon for posts_mentions::Model {
    type Output = Mention;

    fn id(&self) -> Option<Uuid> {
        None
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let account = Accounts::find_by_id(self.account_id)
            .one(state.db_conn)
            .await?
            .expect("[Bug] Mention without associated account");

        let mut acct = account.username.clone();
        if let Some(ref domain) = account.domain {
            acct.push('@');
            acct.push_str(domain);
        }

        Ok(Mention {
            id: account.id,
            acct,
            username: account.username,
            url: account.url,
        })
    }
}

#[async_trait]
impl IntoMastodon for posts::Model {
    type Output = Status;

    fn id(&self) -> Option<Uuid> {
        Some(self.id)
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let account = Accounts::find_by_id(self.account_id)
            .one(state.db_conn)
            .await?
            .expect("[Bug] Post without associated account")
            .into_mastodon(state)
            .await?;

        let reblog_count = Posts::find()
            .filter(posts::Column::RepostedPostId.eq(self.id))
            .count(state.db_conn)
            .await?;

        let favourites_count = self.find_related(Favourites).count(state.db_conn).await?;

        let mentions = PostsMentions::find()
            .filter(posts_mentions::Column::PostId.eq(self.id))
            .stream(state.db_conn)
            .await?
            .map_err(Error::from)
            .and_then(|mention| mention.into_mastodon(state))
            .try_collect()
            .await?;

        let reblog = OptionFuture::from(
            OptionFuture::from(
                self.reposted_post_id
                    .map(|id| Posts::find_by_id(id).one(state.db_conn)),
            )
            .await
            .transpose()?
            .flatten()
            .map(|post| post.into_mastodon(state)),
        )
        .await
        .transpose()?
        .map(Box::new);

        Ok(Status {
            id: self.id,
            created_at: self.created_at.into(),
            in_reply_to_account_id: None,
            in_reply_to_id: self.in_reply_to_id,
            sensitive: self.is_sensitive,
            spoiler_text: self.subject,
            visibility: self.visibility.into(),
            uri: self.url.clone(),
            url: self.url,
            replies_count: 0,
            reblog_count,
            favourites_count,
            content: self.content,
            account,
            media_attachments: Vec::new(),
            mentions,
            reblog,
        })
    }
}
