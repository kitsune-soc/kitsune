use crate::{
    error::{Error, Result},
    service::{attachment::AttachmentService, url::UrlService},
};
use async_trait::async_trait;
use futures_util::{future::OptionFuture, TryStreamExt};
use kitsune_db::{
    model::{
        account::Account as DbAccount,
        favourite::Favourite as DbFavourite,
        media_attachment::{
            MediaAttachment as DbMediaAttachment, PostMediaAttachment as DbPostMediaAttachment,
        },
        mention::Mention as DbMention,
        post::Post as DbPost,
    },
    schema::{accounts, accounts_follows, media_attachments, posts, posts_favourites},
    PgPool,
};
use kitsune_type::mastodon::{
    account::Source, media_attachment::MediaType, relationship::Relationship, status::Mention,
    Account, MediaAttachment, Status,
};
use mime::Mime;
use serde::{de::DeserializeOwned, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct MapperState<'a> {
    pub attachment_service: &'a AttachmentService,
    pub db_conn: &'a PgPool,
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
impl IntoMastodon for DbAccount {
    type Output = Account;

    fn id(&self) -> Option<Uuid> {
        Some(self.id)
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let mut db_conn = state.db_conn.get().await?;

        let (statuses_count, followers_count, following_count) = tokio::try_join!(
            posts::table
                .filter(posts::account_id.eq(self.id))
                .count()
                .get_result(&mut db_conn),
            accounts_follows::table
                .filter(accounts_follows::account_id.eq(self.id))
                .count()
                .get_result(&mut db_conn),
            accounts_follows::table
                .filter(accounts_follows::follower_id.eq(self.id))
                .count()
                .get_result(&mut db_conn)
        );

        let mut acct = self.username.clone();
        if !self.local {
            acct.push('@');
            acct.push_str(&self.domain);
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

        let url = self
            .url
            .unwrap_or_else(|| state.url_service.user_url(self.id));

        Ok(Account {
            id: self.id,
            acct,
            bot: self.actor_type.is_bot(),
            group: self.actor_type.is_group(),
            username: self.username,
            display_name: self.display_name.unwrap_or_default(),
            created_at: self.created_at,
            locked: self.locked,
            note: self.note.unwrap_or_default(),
            url,
            avatar_static: avatar.clone(),
            avatar,
            header_static: header.clone(),
            header,
            followers_count,
            following_count,
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
impl IntoMastodon for (&DbAccount, &DbAccount) {
    type Output = Relationship;

    fn id(&self) -> Option<Uuid> {
        None
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let mut db_conn = state.db_conn.get().await?;

        let (requestor, target) = self;
        let following_requested_fut = accounts_follows::table
            .filter(
                accounts_follows::account_id
                    .eq(requestor.id)
                    .and(accounts_follows::follower_id.eq(target.id)),
            )
            .optional()
            .get_result(&mut db_conn)
            .map_ok(|optional_follow| {
                optional_follow.map_or((false, false), |follow| {
                    (follow.approved_at.is_some(), follow.approved_at.is_none())
                })
            });

        let followed_by_fut = accounts_follows::table
            .filter(
                accounts_follows::account_id
                    .eq(target.id)
                    .and(accounts_follows::follower_id.eq(requestor.id)),
            )
            .count()
            .get_result(&mut db_conn)
            .map_ok(|count| count != 0);

        let ((following, requested), followed_by) =
            tokio::try_join!(following_requested_fut, followed_by_fut)?;

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
impl IntoMastodon for DbMention {
    type Output = Mention;

    fn id(&self) -> Option<Uuid> {
        None
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let mut db_conn = state.db_conn.get().await?;

        let account = accounts::table
            .find(self.account_id)
            .get_result(&mut db_conn)
            .await?
            .expect("[Bug] Mention without associated account");

        let mut acct = account.username.clone();
        if !account.local {
            acct.push('@');
            acct.push_str(&account.domain);
        }

        let url = account
            .url
            .unwrap_or_else(|| state.url_service.user_url(account.id));

        Ok(Mention {
            id: account.id,
            acct,
            username: account.username,
            url,
        })
    }
}

#[async_trait]
impl IntoMastodon for DbMediaAttachment {
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
impl IntoMastodon for (&DbAccount, DbPost) {
    type Output = Status;

    fn id(&self) -> Option<Uuid> {
        None
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let mut db_conn = state.db_conn.get().await?;

        let (account, post) = self;

        let favourited_fut = posts_favourites::table
            .filter(posts_favourites::account_id.eq(account.id))
            .filter(posts_favourites::post_id.eq(post.id))
            .count()
            .get_result(&mut db_conn)
            .map_ok(|count| count != 0);

        let reblogged_fut = posts::table
            .filter(posts::account_id.eq(account.id))
            .filter(posts::reposted_post_id.eq(post.id))
            .count()
            .get_result(&mut db_conn)
            .map_ok(|count| count != 0);

        let (favourited, reblogged) = tokio::try_join!(favourited_fut, reblogged_fut)?;

        let mut status = post.into_mastodon(state).await?;
        status.favourited = favourited;
        status.reblogged = reblogged;

        Ok(status)
    }
}

#[async_trait]
impl IntoMastodon for DbPost {
    type Output = Status;

    fn id(&self) -> Option<Uuid> {
        Some(self.id)
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let mut db_conn = state.db_conn.get().await?;

        let account_fut = accounts::table
            .find(self.account_id)
            .get_result(&mut db_conn)
            .and_then(|db_account| db_account.into_mastodon(state));

        let reblog_count_fut = posts::table
            .filter(posts::reposted_post_id.eq(self.id))
            .count()
            .get_result(&mut db_conn);

        let favourites_count_fut = DbFavourite::belonging_to(&self)
            .count()
            .get_result(&mut db_conn);

        let media_attachments_fut = DbPostMediaAttachment::belonging_to(&self)
            .inner_join(media_attachments::table)
            .select(media_attachments::all_columns)
            .load_stream(&mut db_conn)
            .map_err(Error::from)
            .and_then(|attachment_stream| {
                attachment_stream
                    .and_then(|attachment| attachment.into_mastodon(state))
                    .try_collect()
            })
            .flatten();

        let mentions_stream_fut = DbMention::belonging_to(&self).load_stream(&mut db_conn);

        let reblog = OptionFuture::from(
            OptionFuture::from(
                self.reposted_post_id
                    .map(|id| posts::table.find(id).optional().get_result(&mut db_conn)),
            )
            .await
            .transpose()?
            .flatten()
            .map(|post| post.into_mastodon(state)),
        )
        .await
        .transpose()?
        .map(Box::new);

        let (account, reblog_count, favourites_count, media_attachments, mentions_stream) = tokio::try_join!(
            account_fut,
            reblog_count_fut,
            favourites_count_fut,
            media_attachments_fut,
            mentions_stream_fut,
        );

        let mentions = mentions_stream
            .map_err(Error::from)
            .and_then(|mention| mention.into_mastodon(state))
            .try_collect()
            .await?;

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
