use crate::{error::Result, http::graphql::ContextExt};
use async_graphql::{ComplexObject, Context, SimpleObject};
use chrono::{DateTime, Utc};
use rsa::{pkcs1::DecodeRsaPublicKey, pkcs8::DecodePrivateKey, RsaPrivateKey, RsaPublicKey};
use sea_orm::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq, PartialOrd, SimpleObject)]
#[sea_orm(table_name = "users")]
#[graphql(complex, name = "User")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(indexed)]
    pub username: String,
    #[sea_orm(indexed)]
    pub email: Option<String>,
    #[graphql(skip)]
    pub password: Option<String>,
    #[graphql(skip)]
    pub domain: Option<String>,
    #[sea_orm(indexed, unique)]
    pub url: String,
    #[graphql(skip)]
    pub inbox_url: String,
    #[graphql(skip)]
    pub public_key: Option<String>,
    #[graphql(skip)]
    pub private_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Model {
    pub fn public_key(&self) -> Result<Option<RsaPublicKey>> {
        match (&self.public_key, &self.private_key) {
            (.., Some(private_key)) => {
                let private_key = RsaPrivateKey::from_pkcs8_pem(private_key)?;
                Ok(Some(private_key.to_public_key()))
            }
            (Some(public_key), ..) => Ok(Some(RsaPublicKey::from_pkcs1_pem(public_key)?)),
            (None, None) => {
                error!(user_id = %self.id, "Broken user record encoutered. No key information");
                Ok(None)
            }
        }
    }
}

#[ComplexObject]
impl Model {
    pub async fn posts(&self, ctx: &Context<'_>) -> Result<Vec<super::post::Model>> {
        self.find_related(super::post::Entity)
            .all(&ctx.state().db_conn)
            .await
            .map_err(Into::into)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::post::Entity")]
    Post,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
