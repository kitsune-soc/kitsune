use crate::{
    db::entity::{post, user},
    error::Result,
    state::State,
};
use async_trait::async_trait;
use phenomenon_model::mastodon::{account::Source, Account};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

#[async_trait]
pub trait IntoMastodon {
    type Output;

    async fn into_mastodon(self, state: &State) -> Result<Self::Output>;
}

#[async_trait]
impl IntoMastodon for user::Model {
    type Output = Account;

    async fn into_mastodon(self, state: &State) -> Result<Self::Output> {
        let statuses_count = post::Entity::find()
            .filter(post::Column::UserId.eq(self.id))
            .count(&state.db_conn)
            .await?;
        let mut acct = self.username.clone();
        if let Some(domain) = self.domain {
            acct.push('@');
            acct.push_str(&domain);
        }

        Ok(Account {
            id: self.id,
            acct,
            username: self.username,
            display_name: self.display_name.unwrap_or_default(),
            created_at: self.created_at,
            note: self.note.unwrap_or_default(),
            url: self.url,
            avatar: self.avatar.clone().unwrap_or_else(|| {
                "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into()
            }),
            avatar_static: self.avatar.unwrap_or_else(|| {
                "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into()
            }),
            header: self.header.clone().unwrap_or_default(),
            header_static: self.header.unwrap_or_default(),
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
