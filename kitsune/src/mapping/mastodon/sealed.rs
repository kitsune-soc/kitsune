use crate::{
    error::{Error, Result},
    service::{attachment::AttachmentService, url::UrlService},
};
use async_trait::async_trait;
use futures_util::{future::OptionFuture, TryStreamExt};
use kitsune_db::entity::{
    accounts, accounts_followers, media_attachments, posts, posts_favourites, posts_mentions,
    prelude::{
        Accounts, AccountsFollowers, MediaAttachments, Posts, PostsFavourites, PostsMentions,
    },
};
use kitsune_type::mastodon::{
    account::Source, media_attachment::MediaType, relationship::Relationship, status::Mention,
    Account, MediaAttachment, Status,
};
use mime::Mime;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
};
use serde::{de::DeserializeOwned, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct MapperState<'a> {
    pub attachment_service: &'a AttachmentService,
    pub db_conn: &'a DatabaseConnection,
    pub url_service: &'a UrlService,
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
            state.url_service.default_avatar_url()
        };

        let header = OptionFuture::from(
            self.header_id
                .map(|header_id| state.attachment_service.get_url(header_id)),
        )
        .await
        .transpose()?;

        Ok(Account {
            id: self.id,
            acct,
            bot: self.actor_type.is_bot(),
            username: self.username,
            display_name: self.display_name.unwrap_or_default(),
            created_at: self.created_at,
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

/// Maps to the relationship between the two accounts
///
/// - Left: Requestor of the relationship
/// - Right: Target of the relationship
#[async_trait]
impl IntoMastodon for (&accounts::Model, &accounts::Model) {
    type Output = Relationship;

    fn id(&self) -> Option<Uuid> {
        None
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let (requestor, target) = self;
        let (following, requested) = if let Some(follow) = AccountsFollowers::find()
            .filter(
                accounts_followers::Column::AccountId
                    .eq(requestor.id)
                    .and(accounts_followers::Column::FollowerId.eq(target.id)),
            )
            .one(state.db_conn)
            .await?
        {
            (follow.approved_at.is_some(), follow.approved_at.is_none())
        } else {
            (false, false)
        };

        let followed_by = AccountsFollowers::find()
            .filter(
                accounts_followers::Column::AccountId
                    .eq(target.id)
                    .and(accounts_followers::Column::FollowerId.eq(requestor.id)),
            )
            .count(state.db_conn)
            .await?
            != 0;

        Ok(Relationship {
            id: target.id,
            following,
            showing_reblogs: true,
            notifying: false,
            followed_by,
            blocking: false,
            blocked_by: false,
            muting: false,
            muting_notifications: false,
            requested,
            domain_blocking: false,
            endorsed: false,
            note: String::default(),
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
impl IntoMastodon for media_attachments::Model {
    type Output = MediaAttachment;

    fn id(&self) -> Option<Uuid> {
        Some(self.id)
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let mime_type = Mime::from_str(&self.content_type).unwrap();
        let r#type = match mime_type.type_() {
            mime::AUDIO => MediaType::Audio,
            mime::IMAGE => MediaType::Image,
            mime::VIDEO => MediaType::Video,
            _ => MediaType::Unknown,
        };

        let url = state.attachment_service.get_url(self.id).await?;

        Ok(MediaAttachment {
            id: self.id,
            r#type,
            url: url.clone(),
            preview_url: url.clone(),
            remote_url: url,
            description: self.description.unwrap_or_default(),
            blurhash: self.blurhash,
        })
    }
}

#[async_trait]
impl IntoMastodon for (&accounts::Model, posts::Model) {
    type Output = Status;

    fn id(&self) -> Option<Uuid> {
        None
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let (account, post) = self;

        let favourited = PostsFavourites::find()
            .filter(posts_favourites::Column::AccountId.eq(account.id))
            .filter(posts_favourites::Column::PostId.eq(post.id))
            .count(state.db_conn)
            .await?
            != 0;

        let reblogged = Posts::find()
            .filter(posts::Column::AccountId.eq(account.id))
            .filter(posts::Column::RepostedPostId.eq(post.id))
            .count(state.db_conn)
            .await?
            != 0;

        let mut status = post.into_mastodon(state).await?;
        status.favourited = favourited;
        status.reblogged = reblogged;

        Ok(status)
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

        let favourites_count = self
            .find_related(PostsFavourites)
            .count(state.db_conn)
            .await?;

        let media_attachments = self
            .find_related(MediaAttachments)
            .stream(state.db_conn)
            .await?
            .map_err(Error::from)
            .and_then(|attachment| attachment.into_mastodon(state))
            .try_collect()
            .await?;

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
            created_at: self.created_at,
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
            media_attachments,
            mentions,
            reblog,
            favourited: false,
            reblogged: false,
        })
    }
}
