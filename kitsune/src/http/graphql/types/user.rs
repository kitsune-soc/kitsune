use super::Account;
use crate::http::graphql::ContextExt;
use async_graphql::{ComplexObject, Context, Error, Result, SimpleObject};
use chrono::{DateTime, Utc};
use kitsune_db::entity::{
    prelude::{Accounts, Users},
    users,
};
use sea_orm::{EntityTrait, ModelTrait};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, SimpleObject)]
#[graphql(complex)]
pub struct User {
    pub id: Uuid,
    #[graphql(skip)]
    pub account_id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[ComplexObject]
impl User {
    pub async fn account(&self, ctx: &Context<'_>) -> Result<Option<Account>> {
        let user = Users::find_by_id(self.id)
            .one(&ctx.state().db_conn)
            .await?
            .expect("[Bug] User without associated account");

        user.find_related(Accounts)
            .one(&ctx.state().db_conn)
            .await
            .map(|account| account.map(Into::into))
            .map_err(Error::from)
    }
}

impl From<users::Model> for User {
    fn from(value: users::Model) -> Self {
        Self {
            id: value.id,
            account_id: value.account_id,
            username: value.username,
            email: value.email,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}
