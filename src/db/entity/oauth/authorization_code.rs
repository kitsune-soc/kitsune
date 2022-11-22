use chrono::{DateTime, Utc};
use sea_orm::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "oauth2_authorization_codes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub code: String,
    pub application_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expired_at: DateTime<Utc>,
}

impl Related<super::application::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OAuth2Application.def()
    }
}

impl Related<super::super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::application::Entity",
        from = "Column::ApplicationId",
        to = "super::application::Column::Id"
    )]
    OAuth2Application,

    #[sea_orm(
        belongs_to = "super::super::user::Entity",
        from = "Column::UserId",
        to = "super::super::user::Column::Id"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}
