use crate::{
    error::{ApiError, Error, Result},
    state::Zustand,
    util::BaseToCc,
};
use async_trait::async_trait;
use futures_util::TryStreamExt;
use kitsune_db::{
    column::UrlQuery,
    entity::{
        accounts, media_attachments, posts, posts_mentions,
        prelude::{Accounts, MediaAttachments, PostsMentions},
    },
    link::InReplyTo,
};
use kitsune_type::ap::{
    ap_context,
    helper::StringOrObject,
    object::{Actor, ActorType, MediaAttachment, MediaAttachmentType, PublicKey},
    Object, ObjectType, Tag, TagType,
};
use mime::Mime;
use sea_orm::{prelude::*, QuerySelect};
use std::str::FromStr;

#[async_trait]
pub trait IntoObject {
    type Output;

    async fn into_object(self, state: &Zustand) -> Result<Self::Output>;
}

#[async_trait]
impl IntoObject for media_attachments::Model {
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
impl IntoObject for posts::Model {
    type Output = Object;

    async fn into_object(self, state: &Zustand) -> Result<Self::Output> {
        // Right now a repost can't have content
        // Therefore it's also not an object
        // We just return en error here
        if self.reposted_post_id.is_some() {
            return Err(ApiError::NotFound.into());
        }

        let account = Accounts::find_by_id(self.account_id)
            .one(&state.db_conn)
            .await?
            .expect("[Bug] No user associated with post");

        let in_reply_to = self
            .find_linked(InReplyTo)
            .select_only()
            .column(posts::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?;

        let attachment = self
            .find_related(MediaAttachments)
            .stream(&state.db_conn)
            .await?
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

        let mentions = PostsMentions::find()
            .filter(posts_mentions::Column::PostId.eq(self.id))
            .find_also_related(Accounts)
            .all(&state.db_conn)
            .await?;

        let mut tag = Vec::new();
        let (mut to, cc) = self.visibility.base_to_cc(&account);
        for (mention, mentioned) in mentions {
            let mentioned_url = mentioned.unwrap().url;

            to.push(mentioned_url.clone());
            tag.push(Tag {
                r#type: TagType::Mention,
                name: mention.mention_text,
                href: Some(mentioned_url),
                icon: None,
            });
        }

        Ok(Object {
            context: ap_context(),
            id: self.url,
            r#type: ObjectType::Note,
            attributed_to: StringOrObject::String(account.url),
            in_reply_to,
            sensitive: self.is_sensitive,
            summary: self.subject,
            content: self.content,
            attachment,
            tag,
            published: self.created_at.into(),
            to,
            cc,
        })
    }
}

#[async_trait]
impl IntoObject for accounts::Model {
    type Output = Actor;

    async fn into_object(self, state: &Zustand) -> Result<Self::Output> {
        let public_key_id = format!("{}#main-key", self.url);
        let icon = if let Some(avatar_id) = self.avatar_id {
            let media_attachment = MediaAttachments::find_by_id(avatar_id)
                .one(&state.db_conn)
                .await?
                .expect("[Bug] Missing media attachment");
            Some(media_attachment.into_object(state).await?)
        } else {
            None
        };
        let image = if let Some(header_id) = self.header_id {
            let media_attachment = MediaAttachments::find_by_id(header_id)
                .one(&state.db_conn)
                .await?
                .expect("[Bug] Missing media attachment");
            Some(media_attachment.into_object(state).await?)
        } else {
            None
        };

        // TODO: Save these into the database
        let outbox_url = format!("{}/outbox", self.url);
        let following_url = format!("{}/following", self.url);

        Ok(Actor {
            context: ap_context(),
            id: self.url.clone(),
            r#type: ActorType::Person,
            name: self.display_name,
            subject: self.note,
            icon,
            image,
            preferred_username: self.username,
            manually_approves_followers: self.locked,
            inbox: self.inbox_url,
            outbox: outbox_url,
            followers: self.followers_url,
            following: following_url,
            public_key: PublicKey {
                id: public_key_id,
                owner: self.url,
                public_key_pem: self.public_key,
            },
            published: self.created_at.into(),
        })
    }
}
