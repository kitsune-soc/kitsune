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
            display_name: String::new(),
            created_at: self.created_at,
            note: String::new(),
            url: self.url,
            avatar: "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into(),
            avatar_static: "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into(),
            header: String::new(),
            header_static: String::new(),
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
