use crate::{
    db::entity::{post, user},
    error::{Error, Result},
    state::State,
};
use async_trait::async_trait;
use phenomenon_model::ap::{
    helper::StringOrObject,
    object::{Actor, Note, PublicKey},
    BaseObject, Object,
};
use rsa::{pkcs1::EncodeRsaPublicKey, pkcs8::LineEnding};
use sea_orm::EntityTrait;

#[async_trait]
pub trait IntoActivityPub {
    type Output;

    async fn into_activitypub(self, state: &State) -> Result<Self::Output>;
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

    async fn into_activitypub(self, _state: &State) -> Result<Self::Output> {
        let public_key = self
            .public_key()?
            .ok_or(Error::BrokenRecord)?
            .to_pkcs1_pem(LineEnding::LF)?;

        let public_key_id = format!("{}#main-key", self.url);

        Ok(Object::Person(Actor {
            // TODO: Add avatar and header image to the actor
            name: self.display_name,
            subject: self.note,
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
