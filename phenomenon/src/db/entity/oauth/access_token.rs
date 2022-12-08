use chrono::{DateTime, Duration, Utc};
use sea_orm::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "oauth2_access_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub token: String,
    pub user_id: Option<Uuid>,
    pub application_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub expired_at: DateTime<Utc>,
}

impl Model {
    /// Time until the token is expired
    pub fn ttl(&self) -> Duration {
        self.expired_at - self.created_at
    }
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

    #[sea_orm(has_one = "super::refresh_token::Entity")]
    OAuth2RefreshToken,

    #[sea_orm(
        belongs_to = "super::super::user::Entity",
        from = "Column::UserId",
        to = "super::super::user::Column::Id"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}
