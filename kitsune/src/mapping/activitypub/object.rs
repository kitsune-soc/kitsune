use crate::{
    error::{ApiError, Error, Result},
    state::Zustand,
    try_join,
    util::BaseToCc,
};
use async_trait::async_trait;
use diesel::{BelongingToDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{future::OptionFuture, FutureExt, TryFutureExt, TryStreamExt};
use kitsune_db::{
    model::{
        account::Account,
        media_attachment::{MediaAttachment as DbMediaAttachment, PostMediaAttachment},
        mention::Mention,
        post::Post,
    },
    schema::{accounts, media_attachments, posts},
};
use kitsune_type::ap::{
    actor::{Actor, PublicKey},
    ap_context,
    object::{MediaAttachment, MediaAttachmentType},
    AttributedToField, Object, ObjectType, Tag, TagType,
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
            media_type: Some(self.content_type),
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
            .select(Account::as_select())
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
            .select((Mention::as_select(), Account::as_select()))
            .load::<(Mention, Account)>(&mut db_conn);

        let attachment_stream_fut = PostMediaAttachment::belonging_to(&self)
            .inner_join(media_attachments::table)
            .select(DbMediaAttachment::as_select())
            .load_stream::<DbMediaAttachment>(&mut db_conn);

        let (account, in_reply_to, mentions, attachment_stream) = try_join!(
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
                    media_type: Some(attachment.content_type),
                    url,
                })
            })
            .try_collect()
            .await?;

        let mut tag = Vec::new();
        let (mut to, cc) = self.visibility.base_to_cc(state, &account);
        for (mention, mentioned) in mentions {
            to.push(mentioned.url.clone());
            tag.push(Tag {
                r#type: TagType::Mention,
                name: mention.mention_text,
                href: Some(mentioned.url),
                icon: None,
            });
        }
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

#[async_trait]
impl IntoObject for Account {
    type Output = Actor;

    async fn into_object(self, state: &Zustand) -> Result<Self::Output> {
        let mut db_conn = state.db_conn.get().await?;

        let icon_fut = OptionFuture::from(self.avatar_id.map(|avatar_id| {
            media_attachments::table
                .find(avatar_id)
                .get_result::<DbMediaAttachment>(&mut db_conn)
                .map_err(Error::from)
                .and_then(|media_attachment| media_attachment.into_object(state))
        }))
        .map(Option::transpose);

        let image_fut = OptionFuture::from(self.header_id.map(|header_id| {
            media_attachments::table
                .find(header_id)
                .get_result::<DbMediaAttachment>(&mut db_conn)
                .map_err(Error::from)
                .and_then(|media_attachment| media_attachment.into_object(state))
        }))
        .map(Option::transpose);

        let (icon, image) = try_join!(icon_fut, image_fut)?;

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
