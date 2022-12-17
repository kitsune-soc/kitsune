use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq, SimpleObject)]
#[sea_orm(table_name = "media_attachments")]
#[graphql(name = "MediaAttachment")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub content_type: String,
    pub description: Option<String>,
    pub blurhash: Option<String>,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,
}

impl ActiveModelBehavior for ActiveModel {}
