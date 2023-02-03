use chrono::{DateTime, Utc};
use sea_orm::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "accounts_followers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub follower_id: Uuid,
    pub approved_at: Option<DateTime<Utc>>,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,

    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::FollowerId",
        to = "super::account::Column::Id"
    )]
    Follower,
}

impl ActiveModelBehavior for ActiveModel {}
