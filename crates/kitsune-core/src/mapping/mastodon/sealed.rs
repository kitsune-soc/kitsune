use crate::{
    error::{Error, Result},
    service::{attachment::AttachmentService, url::UrlService},
    try_join,
};
use async_trait::async_trait;
use diesel::{
    BelongingToDsl, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    OptionalExtension, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use futures_util::{future::OptionFuture, FutureExt, TryFutureExt, TryStreamExt};
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    model::{
        account::Account as DbAccount,
        custom_emoji::CustomEmoji as DbCustomEmoji,
        favourite::Favourite as DbFavourite,
        follower::Follow,
        link_preview::LinkPreview,
        media_attachment::{
            MediaAttachment as DbMediaAttachment, PostMediaAttachment as DbPostMediaAttachment,
        },
        mention::Mention as DbMention,
        notification::Notification as DbNotification,
        post::{Post as DbPost, PostSource},
    },
    schema::{
        accounts, accounts_follows, media_attachments, notifications, posts, posts_favourites,
    },
    PgPool,
};
use kitsune_embed::Client as EmbedClient;
use kitsune_embed::{embed_sdk::EmbedType, Embed};
use kitsune_type::mastodon::{
    account::Source,
    media_attachment::MediaType,
    preview_card::PreviewType,
    relationship::Relationship,
    status::{Mention, StatusSource},
    Account, CustomEmoji, MediaAttachment, Notification, PreviewCard, Status,
};
use mime::Mime;
use scoped_futures::ScopedFutureExt;
use serde::{de::DeserializeOwned, Serialize};
use smol_str::SmolStr;
use speedy_uuid::Uuid;
use std::{fmt::Write, str::FromStr};

#[derive(Clone, Copy)]
pub struct MapperState<'a> {
    pub attachment_service: &'a AttachmentService,
    pub db_pool: &'a PgPool,
    pub embed_client: Option<&'a EmbedClient>,
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
        let (statuses_count, followers_count, following_count) = state
            .db_pool
            .with_connection(|db_conn| {
                async {
                    let statuses_count_fut = posts::table
                        .filter(posts::account_id.eq(self.id))
                        .count()
                        .get_result::<i64>(db_conn);

                    let followers_count_fut = accounts_follows::table
                        .filter(accounts_follows::account_id.eq(self.id))
                        .count()
                        .get_result::<i64>(db_conn);

                    let following_count_fut = accounts_follows::table
                        .filter(accounts_follows::follower_id.eq(self.id))
                        .count()
                        .get_result::<i64>(db_conn);

                    try_join!(statuses_count_fut, followers_count_fut, following_count_fut)
                }
                .scoped()
            })
            .await?;

        let mut acct = self.username.clone();
        if !self.local {
            let _ = write!(acct, "@{}", self.domain);
        }

        let avatar = if let Some(avatar_id) = self.avatar_id {
            state.attachment_service.get_url(avatar_id).await?
        } else {
            state.url_service.default_avatar_url()
        };

        let header = if let Some(header_id) = self.header_id {
            state.attachment_service.get_url(header_id).await?
        } else {
            state.url_service.default_header_url()
        };

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
            url: self.url,
            avatar_static: avatar.clone(),
            avatar,
            header_static: header.clone(),
            header,
            followers_count: followers_count as u64,
            following_count: following_count as u64,
            statuses_count: statuses_count as u64,
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
        let (requestor, target) = self;

        let ((following, requested), followed_by) = state
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    let following_requested_fut = accounts_follows::table
                        .filter(
                            accounts_follows::account_id
                                .eq(target.id)
                                .and(accounts_follows::follower_id.eq(requestor.id)),
                        )
                        .get_result::<Follow>(db_conn)
                        .map(OptionalExtension::optional)
                        .map_ok(|optional_follow| {
                            optional_follow.map_or((false, false), |follow| {
                                (follow.approved_at.is_some(), follow.approved_at.is_none())
                            })
                        });

                    let followed_by_fut = accounts_follows::table
                        .filter(
                            accounts_follows::account_id
                                .eq(requestor.id)
                                .and(accounts_follows::follower_id.eq(target.id)),
                        )
                        .count()
                        .get_result::<i64>(db_conn)
                        .map_ok(|count| count != 0);

                    try_join!(following_requested_fut, followed_by_fut)
                }
                .scoped()
            })
            .await?;

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
        let account: DbAccount = state
            .db_pool
            .with_connection(|db_conn| {
                accounts::table
                    .find(self.account_id)
                    .select(DbAccount::as_select())
                    .get_result(db_conn)
                    .scoped()
            })
            .await?;

        let mut acct = account.username.clone();
        if !account.local {
            acct.push('@');
            acct.push_str(&account.domain);
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
        let (account, post) = self;

        let (favourited, reblogged) = state
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    let favourited_fut = posts_favourites::table
                        .filter(posts_favourites::account_id.eq(account.id))
                        .filter(posts_favourites::post_id.eq(post.id))
                        .count()
                        .get_result::<i64>(db_conn)
                        .map_ok(|count| count != 0);

                    let reblogged_fut = posts::table
                        .filter(posts::account_id.eq(account.id))
                        .filter(posts::reposted_post_id.eq(post.id))
                        .count()
                        .get_result::<i64>(db_conn)
                        .map_ok(|count| count != 0);

                    try_join!(favourited_fut, reblogged_fut)
                }
                .scoped()
            })
            .await?;

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

    #[allow(clippy::too_many_lines)]
    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let (account, reblog_count, favourites_count, media_attachments, mentions_stream) = state
            .db_pool
            .with_connection(|db_conn| {
                async {
                    let account_fut = accounts::table
                        .find(self.account_id)
                        .select(DbAccount::as_select())
                        .get_result::<DbAccount>(db_conn)
                        .map_err(Error::from)
                        .and_then(|db_account| db_account.into_mastodon(state));

                    let reblog_count_fut = posts::table
                        .filter(posts::reposted_post_id.eq(self.id))
                        .count()
                        .get_result::<i64>(db_conn)
                        .map_err(Error::from);

                    let favourites_count_fut = DbFavourite::belonging_to(&self)
                        .count()
                        .get_result::<i64>(db_conn)
                        .map_err(Error::from);

                    let media_attachments_fut = DbPostMediaAttachment::belonging_to(&self)
                        .inner_join(media_attachments::table)
                        .select(DbMediaAttachment::as_select())
                        .load_stream::<DbMediaAttachment>(db_conn)
                        .map_err(Error::from)
                        .and_then(|attachment_stream| {
                            attachment_stream
                                .map_err(Error::from)
                                .and_then(|attachment| attachment.into_mastodon(state))
                                .try_collect()
                        });

                    let mentions_stream_fut = DbMention::belonging_to(&self)
                        .load_stream::<DbMention>(db_conn)
                        .map_err(Error::from);

                    try_join!(
                        account_fut,
                        reblog_count_fut,
                        favourites_count_fut,
                        media_attachments_fut,
                        mentions_stream_fut,
                    )
                }
                .scoped()
            })
            .await?;

        let link_preview = OptionFuture::from(
            self.link_preview_url
                .as_ref()
                .and_then(|url| state.embed_client.map(|client| client.fetch_embed(url))),
        )
        .await
        .transpose()?;

        let preview_card =
            OptionFuture::from(link_preview.map(|preview| preview.into_mastodon(state)))
                .await
                .transpose()?;

        let mentions = mentions_stream
            .map_err(Error::from)
            .and_then(|mention| mention.into_mastodon(state))
            .try_collect()
            .await?;

        let reblog = state
            .db_pool
            .with_connection(|db_conn| {
                async {
                    OptionFuture::from(
                        OptionFuture::from(self.reposted_post_id.map(|id| {
                            posts::table
                                .find(id)
                                .select(DbPost::as_select())
                                .get_result::<DbPost>(db_conn)
                                .map(OptionalExtension::optional)
                        }))
                        .await
                        .transpose()?
                        .flatten()
                        .map(|post| post.into_mastodon(state)), // This will allocate two database connections. Fuck.
                    )
                    .await
                    .transpose()
                }
                .scoped()
            })
            .await?
            .map(Box::new);

        let language = self.content_lang.to_639_1().map(str::to_string);

        Ok(Status {
            id: self.id,
            created_at: self.created_at,
            in_reply_to_account_id: None,
            in_reply_to_id: self.in_reply_to_id,
            sensitive: self.is_sensitive,
            spoiler_text: self.subject,
            visibility: self.visibility.into(),
            language,
            uri: self.url.clone(),
            url: self.url,
            replies_count: 0,
            favourites_count: favourites_count as u64,
            reblogs_count: reblog_count as u64,
            content: self.content,
            account,
            media_attachments,
            mentions,
            reblog,
            favourited: false,
            reblogged: false,
            card: preview_card,
        })
    }
}

#[async_trait]
impl IntoMastodon for LinkPreview<Embed> {
    type Output = PreviewCard;

    fn id(&self) -> Option<Uuid> {
        None
    }

    async fn into_mastodon(self, _state: MapperState<'_>) -> Result<Self::Output> {
        let kitsune_db::json::Json(Embed::V1(embed_data)) = self.embed_data else {
            panic!("Incompatible embed data found in database than known to our SDK. Please update Kitsune");
        };

        let title = embed_data.title.unwrap_or_default();
        let description = embed_data.description.unwrap_or_default();
        let (author_name, author_url) = embed_data
            .author
            .map(|author| (author.name, author.url.unwrap_or_default()))
            .unwrap_or_default();

        let (provider_name, provider_url) = (
            embed_data.provider.name.unwrap_or_default(),
            embed_data.provider.url.unwrap_or_default(),
        );

        let r#type = match embed_data.ty {
            EmbedType::Img => PreviewType::Photo,
            EmbedType::Vid => PreviewType::Video,
            _ => PreviewType::Link,
        };

        let image = embed_data.thumb.map(|thumb| thumb.url.clone());
        let (html, width, height, embed_url) = match (embed_data.img, embed_data.video) {
            (.., Some(vid)) => {
                let width = vid.width.unwrap_or_default();
                let height = vid.height.unwrap_or_default();
                let tag_name = if vid.mime.as_deref() == Some(mime::TEXT_HTML.as_ref()) {
                    "iframe"
                } else {
                    "video"
                };

                (
                    format!(
                        r#"<{tag_name} src="{}" width="{width}" height="{height}"></{tag_name}>"#,
                        vid.url,
                    ),
                    width,
                    height,
                    SmolStr::default(),
                )
            }
            (Some(img), ..) => (
                String::default(),
                img.width.unwrap_or_default(),
                img.height.unwrap_or_default(),
                img.url.clone(),
            ),
            _ => (String::default(), 0, 0, SmolStr::default()),
        };

        Ok(PreviewCard {
            url: self.url,
            title,
            description,
            r#type,
            author_name,
            author_url,
            provider_name,
            provider_url,
            html,
            width,
            height,
            image,
            embed_url,
        })
    }
}

#[async_trait]
impl IntoMastodon for PostSource {
    type Output = StatusSource;

    fn id(&self) -> Option<Uuid> {
        None
    }

    async fn into_mastodon(self, _state: MapperState<'_>) -> Result<Self::Output> {
        Ok(StatusSource {
            id: self.id,
            text: self.content,
            spoiler_text: self.subject.unwrap_or_default(),
        })
    }
}

#[async_trait]
impl IntoMastodon for DbNotification {
    type Output = Notification;

    fn id(&self) -> Option<Uuid> {
        None
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let (notification, account, status): (DbNotification, DbAccount, Option<DbPost>) = state
            .db_pool
            .with_connection(|mut db_conn| {
                notifications::table
                    .filter(notifications::receiving_account_id.eq(self.receiving_account_id))
                    .inner_join(
                        accounts::table
                            .on(notifications::triggering_account_id.eq(accounts::id.nullable())),
                    )
                    .left_outer_join(posts::table)
                    .select(<(DbNotification, DbAccount, Option<DbPost>)>::as_select())
                    .get_result(&mut db_conn)
                    .scoped()
            })
            .await?;

        let status = OptionFuture::from(status.map(|status| status.into_mastodon(state)))
            .await
            .transpose()?;

        Ok(Notification {
            id: notification.id,
            r#type: notification.notification_type.into(),
            created_at: notification.created_at,
            account: account.into_mastodon(state).await?,
            status,
        })
    }
}

#[async_trait]
impl IntoMastodon for (DbCustomEmoji, DbMediaAttachment, Option<Timestamp>) {
    type Output = CustomEmoji;

    fn id(&self) -> Option<Uuid> {
        Some(self.0.id)
    }

    async fn into_mastodon(self, state: MapperState<'_>) -> Result<Self::Output> {
        let shortcode = if let Some(ref domain) = self.0.domain {
            format!(":{}@{}:", self.0.shortcode, domain)
        } else {
            format!(":{}:", self.0.shortcode)
        };
        let url = state.url_service.media_url(self.1.id);
        let category = if self.2.is_some() {
            Some(String::from("recently used"))
        } else if self.0.endorsed {
            Some(String::from("endorsed"))
        } else if self.0.domain.is_none() {
            Some(String::from("local"))
        } else {
            Some(self.0.domain.unwrap())
        };
        Ok(CustomEmoji {
            shortcode,
            url: url.clone(),
            static_url: url,
            visible_in_picker: true,
            category,
        })
    }
}
