use super::{util::BaseToCc, State};
use crate::error::{Error, Result};
use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{future::OptionFuture, FutureExt, TryFutureExt, TryStreamExt};
use kitsune_db::{
    model::{
        account::Account,
        custom_emoji::{CustomEmoji, PostCustomEmoji},
        media_attachment::{MediaAttachment as DbMediaAttachment, PostMediaAttachment},
        mention::Mention,
        post::Post,
    },
    schema::{accounts, custom_emojis, media_attachments, posts, posts_custom_emojis},
};
use kitsune_type::ap::{
    actor::{Actor, PublicKey},
    ap_context,
    emoji::Emoji,
    object::{MediaAttachment, MediaAttachmentType},
    AttributedToField, Object, ObjectType, Tag, TagType,
};
use kitsune_util::try_join;
use mime::Mime;
use scoped_futures::ScopedFutureExt;
use std::{future::Future, str::FromStr};

pub trait IntoObject {
    type Output;

    fn into_object(self, state: State<'_>) -> impl Future<Output = Result<Self::Output>> + Send;
}

impl IntoObject for DbMediaAttachment {
    type Output = MediaAttachment;

    async fn into_object(self, state: State<'_>) -> Result<Self::Output> {
        let mime = Mime::from_str(&self.content_type).map_err(|_| Error::UnsupportedMediaType)?;
        let r#type = match mime.type_() {
            mime::AUDIO => MediaAttachmentType::Audio,
            mime::IMAGE => MediaAttachmentType::Image,
            mime::VIDEO => MediaAttachmentType::Video,
            _ => return Err(Error::UnsupportedMediaType),
        };
        let url = state.service.attachment.get_url(self.id).await?;

        Ok(MediaAttachment {
            r#type,
            name: self.description,
            media_type: Some(self.content_type),
            blurhash: self.blurhash,
            url,
        })
    }
}

fn build_post_tags(
    mentions: Vec<(Mention, Account)>,
    to: &mut Vec<String>,
    emojis: Vec<(CustomEmoji, PostCustomEmoji, DbMediaAttachment)>,
) -> Vec<Tag> {
    let mut tag = Vec::new();
    for (mention, mentioned) in mentions {
        to.push(mentioned.url.clone());
        tag.push(Tag {
            id: None,
            r#type: TagType::Mention,
            name: mention.mention_text,
            href: Some(mentioned.url),
            icon: None,
        });
    }
    for (custom_emoji, post_emoji, attachment) in emojis {
        if let Some(attachment_url) = attachment.remote_url {
            tag.push(Tag {
                id: Some(custom_emoji.remote_id),
                r#type: TagType::Emoji,
                name: post_emoji.emoji_text,
                href: None,
                icon: Some(MediaAttachment {
                    r#type: MediaAttachmentType::Image,
                    name: None,
                    media_type: Some(attachment.content_type),
                    blurhash: None,
                    url: attachment_url,
                }),
            });
        }
    }
    tag
}

impl IntoObject for Post {
    type Output = Object;

    async fn into_object(self, state: State<'_>) -> Result<Self::Output> {
        // Right now a repost can't have content
        // Therefore it's also not an object
        // We just return en error here
        if self.reposted_post_id.is_some() {
            return Err(Error::NotFound);
        }

        let (account, in_reply_to, mentions, emojis, attachment_stream) = state
            .db_pool
            .with_connection(|db_conn| {
                async {
                    let account_fut = accounts::table
                        .find(self.account_id)
                        .select(Account::as_select())
                        .get_result(db_conn);

                    let in_reply_to_fut =
                        OptionFuture::from(self.in_reply_to_id.map(|in_reply_to_id| {
                            posts::table
                                .find(in_reply_to_id)
                                .select(posts::url)
                                .get_result(db_conn)
                        }))
                        .map(Option::transpose);

                    let mentions_fut = Mention::belonging_to(&self)
                        .inner_join(accounts::table)
                        .select((Mention::as_select(), Account::as_select()))
                        .load::<(Mention, Account)>(db_conn);

                    let custom_emojis_fut = custom_emojis::table
                        .inner_join(posts_custom_emojis::table)
                        .inner_join(media_attachments::table)
                        .filter(posts_custom_emojis::post_id.eq(self.id))
                        .select((
                            CustomEmoji::as_select(),
                            PostCustomEmoji::as_select(),
                            DbMediaAttachment::as_select(),
                        ))
                        .load::<(CustomEmoji, PostCustomEmoji, DbMediaAttachment)>(db_conn);

                    let attachment_stream_fut = PostMediaAttachment::belonging_to(&self)
                        .inner_join(media_attachments::table)
                        .select(DbMediaAttachment::as_select())
                        .load_stream::<DbMediaAttachment>(db_conn);

                    try_join!(
                        account_fut,
                        in_reply_to_fut,
                        mentions_fut,
                        custom_emojis_fut,
                        attachment_stream_fut
                    )
                }
                .scoped()
            })
            .await?;

        let attachment = attachment_stream
            .map_err(Error::from)
            .and_then(|attachment| async move {
                let url = state.service.attachment.get_url(attachment.id).await?;

                Ok(MediaAttachment {
                    r#type: MediaAttachmentType::Document,
                    name: attachment.description,
                    blurhash: attachment.blurhash,
                    media_type: Some(attachment.content_type),
                    url,
                })
            })
            .try_collect()
            .await?;

        let (mut to, cc) = self.visibility.base_to_cc(state, &account);
        let tag = build_post_tags(mentions, &mut to, emojis);

        let account_url = state.service.url.user_url(account.id);

        Ok(Object {
            context: ap_context(),
            id: self.url,
            r#type: ObjectType::Note,
            attributed_to: AttributedToField::Url(account_url),
            in_reply_to,
            sensitive: self.is_sensitive,
            name: None,
            summary: self.subject,
            content: self.content,
            media_type: None,
            attachment,
            tag,
            published: self.created_at,
            to,
            cc,
        })
    }
}

impl IntoObject for Account {
    type Output = Actor;

    async fn into_object(self, state: State<'_>) -> Result<Self::Output> {
        let (icon, image) = state
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    // These calls also probably allocate two cocnnections. ugh.
                    let icon_fut = OptionFuture::from(self.avatar_id.map(|avatar_id| {
                        media_attachments::table
                            .find(avatar_id)
                            .get_result::<DbMediaAttachment>(db_conn)
                            .map_err(Error::from)
                            .and_then(|media_attachment| media_attachment.into_object(state))
                    }))
                    .map(Option::transpose);

                    let image_fut = OptionFuture::from(self.header_id.map(|header_id| {
                        media_attachments::table
                            .find(header_id)
                            .get_result::<DbMediaAttachment>(db_conn)
                            .map_err(Error::from)
                            .and_then(|media_attachment| media_attachment.into_object(state))
                    }))
                    .map(Option::transpose);

                    try_join!(icon_fut, image_fut)
                }
                .scoped()
            })
            .await?;

        let user_url = state.service.url.user_url(self.id);
        let inbox = state.service.url.inbox_url(self.id);
        let outbox = state.service.url.outbox_url(self.id);
        let followers = state.service.url.followers_url(self.id);
        let following = state.service.url.following_url(self.id);

        Ok(Actor {
            context: ap_context(),
            id: user_url.clone(),
            r#type: self.actor_type.into(),
            name: self.display_name,
            subject: self.note,
            icon,
            image,
            preferred_username: self.username,
            manually_approves_followers: self.locked,
            endpoints: None,
            inbox,
            outbox: Some(outbox),
            featured: None,
            followers: Some(followers),
            following: Some(following),
            public_key: PublicKey {
                id: self.public_key_id,
                owner: user_url,
                public_key_pem: self.public_key,
            },
            published: self.created_at,
        })
    }
}

impl IntoObject for CustomEmoji {
    type Output = Emoji;

    async fn into_object(self, state: State<'_>) -> Result<Self::Output> {
        // Officially we don't have any info about remote emojis as we're not the origin
        // Let's pretend we're not home and do not answer
        let name = match self.domain {
            None => Ok(format!(":{}:", self.shortcode)),
            Some(_) => Err(Error::NotFound),
        }?;

        let icon = state
            .db_pool
            .with_connection(|db_conn| {
                media_attachments::table
                    .find(self.media_attachment_id)
                    .get_result::<DbMediaAttachment>(db_conn)
                    .map_err(Error::from)
                    .and_then(|media_attachment| media_attachment.into_object(state))
                    .scoped()
            })
            .await?;

        Ok(Emoji {
            context: ap_context(),
            id: self.remote_id,
            r#type: String::from("Emoji"),
            name,
            icon,
            updated: self.updated_at,
        })
    }
}
