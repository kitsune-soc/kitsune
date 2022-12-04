use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq, SimpleObject)]
#[sea_orm(table_name = "media_attachments")]
#[graphql(name = "MediaAttachment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub content_type: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
