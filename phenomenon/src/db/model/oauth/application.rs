use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq, SimpleObject)]
#[graphql(name = "OAuthApplication")]
#[sea_orm(table_name = "oauth2_applications")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    #[sea_orm(unique)]
    pub secret: String,
    pub redirect_uri: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::access_token::Entity")]
    OAuth2AccessToken,

    #[sea_orm(has_many = "super::refresh_token::Entity")]
    OAuth2RefreshToken,
}

impl ActiveModelBehavior for ActiveModel {}
