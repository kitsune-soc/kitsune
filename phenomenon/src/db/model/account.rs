use super::media_attachment;
use crate::{error::Result, http::graphql::ContextExt};
use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, DeriveEntityModel, Deserialize, Eq, PartialEq, PartialOrd, Serialize, SimpleObject,
)]
#[sea_orm(table_name = "accounts")]
#[graphql(complex, name = "Account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    // TODO: Express relationship in trait form
    #[graphql(skip)]
    pub avatar_id: Option<Uuid>,
    #[graphql(skip)]
    pub header_id: Option<Uuid>,
    pub display_name: Option<String>,
    pub note: Option<String>,
    #[sea_orm(indexed)]
    pub username: String,
    #[graphql(skip)]
    pub domain: Option<String>,
    #[sea_orm(indexed, unique)]
    pub url: String,
    #[graphql(skip)]
    pub followers_url: String,
    #[graphql(skip)]
    pub inbox_url: String,
    #[graphql(skip)]
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[ComplexObject]
impl Model {
    pub async fn avatar(&self, ctx: &Context<'_>) -> Result<Option<media_attachment::Model>> {
        if let Some(avatar_id) = self.avatar_id {
            media_attachment::Entity::find_by_id(avatar_id)
                .one(&ctx.state().db_conn)
                .await
                .map_err(Into::into)
        } else {
            Ok(None)
        }
    }

    pub async fn header(&self, ctx: &Context<'_>) -> Result<Option<media_attachment::Model>> {
        if let Some(header_id) = self.header_id {
            media_attachment::Entity::find_by_id(header_id)
                .one(&ctx.state().db_conn)
                .await
                .map_err(Into::into)
        } else {
            Ok(None)
        }
    }

    pub async fn posts(&self, ctx: &Context<'_>) -> Result<Vec<super::post::Model>> {
        self.find_related(super::post::Entity)
            .all(&ctx.state().db_conn)
            .await
            .map_err(Into::into)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::media_attachment::Entity")]
    MediaAttachment,

    #[sea_orm(has_many = "super::post::Entity")]
    Post,

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::Id",
        to = "super::user::Column::AccountId"
    )]
    User,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
