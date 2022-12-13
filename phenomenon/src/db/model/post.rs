use crate::http::graphql::ContextExt;
use async_graphql::{ComplexObject, Context, Enum, Result, SimpleObject};
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Enum,
    EnumIter,
    DeriveActiveEnum,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    Public = 0,
    Unlisted = 1,
    FollowerOnly = 2,
    MentionOnly = 3,
}

#[derive(
    Clone, Debug, DeriveEntityModel, Deserialize, Eq, PartialEq, PartialOrd, Serialize, SimpleObject,
)]
#[sea_orm(table_name = "posts")]
#[graphql(complex, name = "Post")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[graphql(skip)]
    pub user_id: Uuid,
    pub is_sensitive: bool,
    #[sea_orm(nullable)]
    pub subject: Option<String>,
    pub content: String,
    pub visibility: Visibility,
    #[sea_orm(unique)]
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[ComplexObject]
impl Model {
    pub async fn user(&self, ctx: &Context<'_>) -> Result<super::user::Model> {
        Ok(super::user::Entity::find_by_id(self.user_id)
            .one(&ctx.state().db_conn)
            .await?
            .expect("[Bug] Post without associated user encountered"))
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
