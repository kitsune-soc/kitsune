use crate::{
    db::entity::{media_attachment, post, user},
    error::{Error, Result},
    state::State,
};
use async_trait::async_trait;
use mime::Mime;
use phenomenon_model::ap::{
    helper::StringOrObject,
    object::{Actor, MediaAttachment, MediaAttachmentType, Note, PublicKey},
    BaseObject, Object,
};
use rsa::{pkcs1::EncodeRsaPublicKey, pkcs8::LineEnding};
use sea_orm::EntityTrait;
use std::str::FromStr;

#[async_trait]
pub trait IntoActivityPub {
    type Output;

    async fn into_activitypub(self, state: &State) -> Result<Self::Output>;
}

#[async_trait]
impl IntoActivityPub for media_attachment::Model {
    type Output = MediaAttachment;

    async fn into_activitypub(self, _state: &State) -> Result<Self::Output> {
        let mime = Mime::from_str(&self.content_type).map_err(|_| Error::UnsupportedMediaType)?;
        let r#type = match mime.type_() {
            mime::AUDIO => MediaAttachmentType::Audio,
            mime::IMAGE => MediaAttachmentType::Image,
            mime::VIDEO => MediaAttachmentType::Video,
            _ => return Err(Error::UnsupportedMediaType),
        };

        Ok(MediaAttachment {
            r#type,
            media_type: self.content_type,
            url: self.url,
        })
    }
}

#[async_trait]
impl IntoActivityPub for post::Model {
    type Output = Object;

    async fn into_activitypub(self, state: &State) -> Result<Self::Output> {
        let user = user::Entity::find_by_id(self.user_id)
            .one(&state.db_conn)
            .await?
            .expect("[Bug] No user associated with post");

        Ok(Object::Note(Note {
            subject: self.subject,
            content: self.content,
            rest: BaseObject {
                id: self.url,
                attributed_to: Some(StringOrObject::String(user.url).into()),
                published_at: self.created_at,
                ..BaseObject::default()
            },
        }))
    }
}

#[async_trait]
impl IntoActivityPub for user::Model {
    type Output = Object;

    async fn into_activitypub(self, state: &State) -> Result<Self::Output> {
        let public_key = self
            .public_key()?
            .ok_or(Error::BrokenRecord)?
            .to_pkcs1_pem(LineEnding::LF)?;

        let public_key_id = format!("{}#main-key", self.url);
        let icon = if let Some(avatar_id) = self.avatar_id {
            let media_attachment = media_attachment::Entity::find_by_id(avatar_id)
                .one(&state.db_conn)
                .await?
                .expect("[Bug] Missing media attachment");
            Some(media_attachment.into_activitypub(state).await?)
        } else {
            None
        };
        let image = if let Some(header_id) = self.header_id {
            let media_attachment = media_attachment::Entity::find_by_id(header_id)
                .one(&state.db_conn)
                .await?
                .expect("[Bug] Missing media attachment");
            Some(media_attachment.into_activitypub(state).await?)
        } else {
            None
        };

        Ok(Object::Person(Actor {
            name: self.display_name,
            subject: self.note,
            icon,
            image,
            preferred_username: self.username,
            inbox: self.inbox_url,
            rest: BaseObject {
                id: self.url.clone(),
                ..BaseObject::default()
            },
            public_key: PublicKey {
                id: public_key_id,
                owner: self.url,
                public_key_pem: public_key,
            },
            ..Actor::default()
        }))
    }
}
