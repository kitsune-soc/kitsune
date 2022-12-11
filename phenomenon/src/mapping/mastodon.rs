use crate::{
    db::model::{media_attachment, post, user},
    error::Result,
    state::Zustand,
};
use async_trait::async_trait;
use phenomenon_model::mastodon::{account::Source, Account};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

#[async_trait]
pub trait IntoMastodon {
    type Output;

    async fn into_mastodon(self, state: &Zustand) -> Result<Self::Output>;
}

#[async_trait]
impl IntoMastodon for user::Model {
    type Output = Account;

    async fn into_mastodon(self, state: &Zustand) -> Result<Self::Output> {
        let statuses_count = post::Entity::find()
            .filter(post::Column::UserId.eq(self.id))
            .count(&state.db_conn)
            .await?;
        let mut acct = self.username.clone();
        if let Some(domain) = self.domain {
            acct.push('@');
            acct.push_str(&domain);
        }

        let avatar = if let Some(avatar_id) = self.avatar_id {
            let media_attachment = media_attachment::Entity::find_by_id(avatar_id)
                .one(&state.db_conn)
                .await?
                .expect("[Bug] User profile picture missing");
            media_attachment.url
        } else {
            "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into()
        };

        let header = if let Some(header_id) = self.header_id {
            let media_attachment = media_attachment::Entity::find_by_id(header_id)
                .one(&state.db_conn)
                .await?
                .expect("[Bug] User header image missing");
            media_attachment.url
        } else {
            "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into()
        };

        Ok(Account {
            id: self.id,
            acct,
            username: self.username,
            display_name: self.display_name.unwrap_or_default(),
            created_at: self.created_at,
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
