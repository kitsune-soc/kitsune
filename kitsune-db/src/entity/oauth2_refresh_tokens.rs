//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.7

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "oauth2_refresh_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub token: String,
    #[sea_orm(column_type = "Text", unique)]
    pub access_token: String,
    pub application_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::oauth2_access_tokens::Entity",
        from = "Column::AccessToken",
        to = "super::oauth2_access_tokens::Column::Token",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Oauth2AccessTokens,
    #[sea_orm(
        belongs_to = "super::oauth2_applications::Entity",
        from = "Column::ApplicationId",
        to = "super::oauth2_applications::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Oauth2Applications,
}

impl Related<super::oauth2_access_tokens::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Oauth2AccessTokens.def()
    }
}

impl Related<super::oauth2_applications::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Oauth2Applications.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
