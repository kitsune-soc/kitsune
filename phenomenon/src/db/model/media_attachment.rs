use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq, SimpleObject)]
#[sea_orm(table_name = "media_attachments")]
#[graphql(name = "MediaAttachment")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub content_type: String,
    pub description: Option<String>,
    pub blurhash: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
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

impl ActiveModelBehavior for ActiveModel {}
