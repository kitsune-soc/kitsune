use crate::{
    error::{ApiError, Error, Result},
    state::Zustand,
    util::BaseToCc,
};
use async_trait::async_trait;
use diesel::{BelongingToDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{future::OptionFuture, FutureExt, TryStreamExt};
use kitsune_db::{
    model::{
        account::Account,
        media_attachment::{MediaAttachment as DbMediaAttachment, PostMediaAttachment},
        mention::Mention,
        post::Post,
    },
    schema::{accounts, media_attachments, posts, posts_mentions},
};
use kitsune_type::ap::{
    actor::{Actor, PublicKey},
    ap_context,
    helper::StringOrObject,
    object::{MediaAttachment, MediaAttachmentType},
    Object, ObjectType, Tag, TagType,
};
use mime::Mime;
use std::str::FromStr;

#[async_trait]
pub trait IntoObject {
    type Output;

    async fn into_object(self, state: &Zustand) -> Result<Self::Output>;
}

#[async_trait]
impl IntoObject for DbMediaAttachment {
    type Output = MediaAttachment;

    async fn into_object(self, state: &Zustand) -> Result<Self::Output> {
        let mime =
            Mime::from_str(&self.content_type).map_err(|_| ApiError::UnsupportedMediaType)?;
        let r#type = match mime.type_() {
            mime::AUDIO => MediaAttachmentType::Audio,
            mime::IMAGE => MediaAttachmentType::Image,
            mime::VIDEO => MediaAttachmentType::Video,
            _ => return Err(ApiError::UnsupportedMediaType.into()),
        };
        let url = state.service.attachment.get_url(self.id).await?;

        Ok(MediaAttachment {
            r#type,
            name: self.description,
            media_type: self.content_type,
            blurhash: self.blurhash,
            url,
        })
    }
}

#[async_trait]
impl IntoObject for Post {
    type Output = Object;

    async fn into_object(self, state: &Zustand) -> Result<Self::Output> {
        // Right now a repost can't have content
        // Therefore it's also not an object
        // We just return en error here
        if self.reposted_post_id.is_some() {
            return Err(ApiError::NotFound.into());
        }

        let mut db_conn = state.db_conn.get().await?;
        let account_fut = accounts::table
            .find(self.account_id)
            .select(Account::columns())
            .get_result(&mut db_conn);

        let in_reply_to_fut = OptionFuture::from(self.in_reply_to_id.map(|in_reply_to_id| {
            posts::table
                .find(in_reply_to_id)
                .select(posts::url)
                .get_result(&mut db_conn)
        }))
        .map(Option::transpose);

        let mentions_fut = Mention::belonging_to(&self)
            .inner_join(accounts::table)
            .select((posts_mentions::all_columns, Account::as_select()))
            .load::<(Mention, Account)>(&mut db_conn);

        let attachment_stream_fut = PostMediaAttachment::belonging_to(&self)
            .inner_join(media_attachments::table)
            .select(media_attachments::all_columns)
            .load_stream::<DbMediaAttachment>(&mut db_conn);

        let (account, in_reply_to, mentions, attachment_stream) = tokio::try_join!(
            account_fut,
            in_reply_to_fut,
            mentions_fut,
            attachment_stream_fut
        )?;

        let attachment = attachment_stream
            .map_err(Error::from)
            .and_then(|attachment| async move {
                let url = state.service.attachment.get_url(attachment.id).await?;

                Ok(MediaAttachment {
                    r#type: MediaAttachmentType::Document,
                    name: attachment.description,
                    blurhash: attachment.blurhash,
                    media_type: attachment.content_type,
                    url,
                })
            })
            .try_collect()
            .await?;

        let mut tag = Vec::new();
        let (mut to, cc) = self.visibility.base_to_cc(state, &account);
        for (mention, mentioned) in mentions {
            let mentioned_url = mentioned
                .url
                .unwrap_or_else(|| state.service.url.user_url(account.id));

            to.push(mentioned_url.clone());
            tag.push(Tag {
                r#type: TagType::Mention,
                name: mention.mention_text,
                href: Some(mentioned_url),
                icon: None,
            });
        }
        let account_url = state.service.url.user_url(account.id);

        Ok(Object {
            context: ap_context(),
            id: self.url,
            r#type: ObjectType::Note,
            attributed_to: StringOrObject::String(account_url),
            in_reply_to,
            sensitive: self.is_sensitive,
            summary: self.subject,
            content: self.content,
            attachment,
            tag,
            published: self.created_at,
            to,
            cc,
        })
    }
}

#[async_trait]
impl IntoObject for Account {
    type Output = Actor;

    async fn into_object(self, state: &Zustand) -> Result<Self::Output> {
        let mut db_conn = state.db_conn.get().await?;
        let icon = if let Some(avatar_id) = self.avatar_id {
            let media_attachment = media_attachments::table
                .find(avatar_id)
                .get_result::<DbMediaAttachment>(&mut db_conn)
                .await?;

            Some(media_attachment.into_object(state).await?)
        } else {
            None
        };
        let image = if let Some(header_id) = self.header_id {
            let media_attachment = media_attachments::table
                .find(header_id)
                .get_result::<DbMediaAttachment>(&mut db_conn)
                .await?;

            Some(media_attachment.into_object(state).await?)
        } else {
            None
        };

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
            outbox,
            featured: None,
            followers,
            following,
            public_key: PublicKey {
                id: self.public_key_id,
                owner: user_url,
                public_key_pem: self.public_key,
            },
            published: self.created_at,
        })
    }
}
