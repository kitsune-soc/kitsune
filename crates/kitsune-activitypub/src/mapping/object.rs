use super::{State, util::BaseToCc};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{FutureExt, TryFutureExt, TryStreamExt, future::OptionFuture};
use kitsune_db::{
    model::{
        Account, CustomEmoji, MediaAttachment as DbMediaAttachment, Post, PostsCustomEmoji,
        PostsMention,
    },
    schema::{
        accounts, accounts_activitypub, cryptographic_keys, custom_emojis, media_attachments,
        posts, posts_custom_emojis, posts_media_attachments, posts_mentions,
    },
    with_connection,
};
use kitsune_error::{Error, ErrorType, Result, bail, kitsune_error};
use kitsune_type::ap::{
    Object, ObjectType, Tag, TagType,
    actor::{Actor, PublicKey},
    ap_context,
    emoji::Emoji,
    object::{MediaAttachment, MediaAttachmentType},
};
use kitsune_util::try_join;
use mime::Mime;
use std::str::FromStr;

pub trait IntoObject {
    type Output;

    fn into_object(self, state: State<'_>) -> impl Future<Output = Result<Self::Output>> + Send;
}

impl IntoObject for DbMediaAttachment {
    type Output = MediaAttachment;

    async fn into_object(self, state: State<'_>) -> Result<Self::Output> {
        let mime = Mime::from_str(&self.content_type).map_err(
            |_| kitsune_error!(type = ErrorType::UnsupportedMediaType, "unsupported media type"),
        )?;

        let r#type = match mime.type_() {
            mime::AUDIO => MediaAttachmentType::Audio,
            mime::IMAGE => MediaAttachmentType::Image,
            mime::VIDEO => MediaAttachmentType::Video,
            _ => {
                return Err(
                    kitsune_error!(type = ErrorType::UnsupportedMediaType, "unsupported media type"),
                );
            }
        };
        let url = state.service.attachment.get_url(self.id).await?;

        Ok(MediaAttachment {
            r#type,
            name: self.description,
            media_type: Some(self.content_type),
            blurhash: None,
            url,
        })
    }
}

fn build_post_tags(
    mentions: Vec<(PostsMention, Account)>,
    to: &mut Vec<String>,
    emojis: Vec<(CustomEmoji, PostsCustomEmoji, DbMediaAttachment)>,
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
            bail!("post not found");
        }

        let (account, in_reply_to, mentions, emojis, attachment_stream) =
            with_connection!(state.db_pool, |db_conn| {
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

                let mentions_fut = posts_mentions::table
                    .filter(posts_mentions::post_id.eq(self.id))
                    .inner_join(accounts::table.on(posts_mentions::account_id.eq(accounts::id)))
                    .select((PostsMention::as_select(), Account::as_select()))
                    .load::<(PostsMention, Account)>(db_conn);

                let custom_emojis_fut = custom_emojis::table
                    .inner_join(posts_custom_emojis::table)
                    .inner_join(media_attachments::table)
                    .filter(posts_custom_emojis::post_id.eq(self.id))
                    .select((
                        CustomEmoji::as_select(),
                        PostsCustomEmoji::as_select(),
                        DbMediaAttachment::as_select(),
                    ))
                    .load::<(CustomEmoji, PostsCustomEmoji, DbMediaAttachment)>(db_conn);

                let attachment_stream_fut =
                    posts_media_attachments::table
                        .filter(posts_media_attachments::post_id.eq(self.id))
                        .inner_join(media_attachments::table.on(
                            posts_media_attachments::media_attachment_id.eq(media_attachments::id),
                        ))
                        .select(DbMediaAttachment::as_select())
                        .load_stream::<DbMediaAttachment>(db_conn);

                try_join!(
                    account_fut,
                    in_reply_to_fut,
                    mentions_fut,
                    custom_emojis_fut,
                    attachment_stream_fut
                )
            })?;

        let attachment = attachment_stream
            .map_err(Error::from)
            .and_then(|attachment| async move {
                let url = state.service.attachment.get_url(attachment.id).await?;

                Ok(MediaAttachment {
                    r#type: MediaAttachmentType::Document,
                    name: attachment.description,
                    blurhash: None,
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
            attributed_to: account_url,
            in_reply_to,
            sensitive: false,
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
        let (icon, image, public_key_info) = with_connection!(state.db_pool, |db_conn| {
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

            let public_key_fut = accounts_activitypub::table
                .filter(accounts_activitypub::account_id.eq(self.id))
                .inner_join(
                    cryptographic_keys::table
                        .on(accounts_activitypub::key_id.eq(cryptographic_keys::key_id)),
                )
                .select((
                    accounts_activitypub::key_id,
                    cryptographic_keys::public_key_der,
                ))
                .first::<(String, Vec<u8>)>(db_conn)
                .map_err(Error::from);

            try_join!(icon_fut, image_fut, public_key_fut)
        })?;

        let user_url = state.service.url.user_url(self.id);
        let inbox = state.service.url.inbox_url(self.id);
        let outbox = state.service.url.outbox_url(self.id);
        let followers = state.service.url.followers_url(self.id);
        let following = state.service.url.following_url(self.id);

        Ok(Actor {
            context: ap_context(),
            id: user_url.clone(),
            r#type: self.account_type.into(),
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
                id: public_key_info.0,
                owner: user_url,
                public_key_pem: String::from_utf8_lossy(&public_key_info.1).to_string(),
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
            Some(_) => Err(kitsune_error!("custom emoji not found")),
        }?;

        let icon = with_connection!(state.db_pool, |db_conn| {
            media_attachments::table
                .find(self.media_attachment_id)
                .get_result::<DbMediaAttachment>(db_conn)
                .map_err(Error::from)
                .and_then(|media_attachment| media_attachment.into_object(state))
                .await
        })?;

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
