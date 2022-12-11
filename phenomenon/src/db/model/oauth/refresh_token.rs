use chrono::{DateTime, Utc};
use sea_orm::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "oauth2_refresh_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub token: String,
    pub access_token: String,
    pub application_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Related<super::access_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OAuth2AccessToken.def()
    }
}

impl Related<super::application::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OAuth2Application.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::access_token::Entity",
        from = "Column::AccessToken",
        to = "super::access_token::Column::Token"
    )]
    OAuth2AccessToken,

    #[sea_orm(
        belongs_to = "super::application::Entity",
        from = "Column::ApplicationId",
        to = "super::application::Column::Id"
    )]
    OAuth2Application,
}

impl ActiveModelBehavior for ActiveModel {}
