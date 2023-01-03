use chrono::{DateTime, Utc};
use clap::ValueEnum;
use sea_orm::prelude::*;
use uuid::Uuid;

#[derive(
    Copy, Clone, Debug, DeriveActiveEnum, EnumIter, Eq, Ord, PartialEq, PartialOrd, ValueEnum,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Role {
    Admin = 0,
}

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "users_roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: Role,
    pub created_at: DateTime<Utc>,
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
