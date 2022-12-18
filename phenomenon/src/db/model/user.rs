use crate::http::graphql::ContextExt;
use async_graphql::{ComplexObject, Context, Error, Result, SimpleObject};
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone, Debug, DeriveEntityModel, Deserialize, Eq, PartialEq, PartialOrd, Serialize, SimpleObject,
)]
#[sea_orm(table_name = "users")]
#[graphql(complex, name = "User")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[graphql(skip)]
    #[sea_orm(unique)]
    pub account_id: Uuid,
    #[sea_orm(indexed)]
    pub username: String,
    #[sea_orm(indexed)]
    pub email: String,
    #[graphql(skip)]
    pub password: String,
    #[graphql(skip)]
    pub private_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[ComplexObject]
impl Model {
    pub async fn account(&self, ctx: &Context<'_>) -> Result<Option<super::account::Model>> {
        self.find_related(super::account::Entity)
            .one(&ctx.state().db_conn)
            .await
            .map_err(Error::from)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::account::Entity")]
    Account,

    #[sea_orm(has_many = "super::oauth::access_token::Entity")]
    OAuth2AccessToken,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
