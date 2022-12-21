use super::account;
use crate::http::graphql::ContextExt;
use async_graphql::{ComplexObject, Context, Enum, Result, SimpleObject};
use chrono::{DateTime, Utc};
use phenomenon_type::ap::{helper::CcTo, Privacy};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Enum,
    EnumIter,
    DeriveActiveEnum,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    Public = 0,
    Unlisted = 1,
    FollowerOnly = 2,
    MentionOnly = 3,
}

impl Visibility {
    pub fn from_activitypub<O>(owner: &account::Model, obj: &O) -> Self
    where
        O: CcTo + Privacy,
    {
        if obj.is_public() {
            Self::Public
        } else if obj.is_unlisted() {
            Self::Unlisted
        } else if obj.to().contains(&owner.followers_url) {
            Self::FollowerOnly
        } else {
            Self::MentionOnly
        }
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn json_repr(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(
    Clone, Debug, DeriveEntityModel, Deserialize, Eq, PartialEq, PartialOrd, Serialize, SimpleObject,
)]
#[sea_orm(table_name = "posts")]
#[graphql(complex, name = "Post")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[graphql(skip)]
    pub account_id: Uuid,
    pub is_sensitive: bool,
    #[sea_orm(nullable)]
    pub subject: Option<String>,
    pub content: String,
    pub visibility: Visibility,
    #[sea_orm(unique)]
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[ComplexObject]
impl Model {
    pub async fn account(&self, ctx: &Context<'_>) -> Result<super::account::Model> {
        Ok(super::account::Entity::find_by_id(self.account_id)
            .one(&ctx.state().db_conn)
            .await?
            .expect("[Bug] Post without associated user encountered"))
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

    #[sea_orm(has_many = "super::favourite::Entity")]
    Favourite,

    #[sea_orm(has_many = "super::mention::Entity")]
    Mention,

    #[sea_orm(has_many = "super::repost::Entity")]
    Repost,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::favourite::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Favourite.def()
    }
}

impl Related<super::mention::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Mention.def()
    }
}

impl Related<super::repost::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repost.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
