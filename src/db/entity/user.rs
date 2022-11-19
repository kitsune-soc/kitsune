use crate::error::Result;
use chrono::{DateTime, Utc};
use rsa::{pkcs8::DecodePrivateKey, RsaPrivateKey, RsaPublicKey};
use sea_orm::prelude::*;
use uuid::Uuid;
use zeroize::Zeroize;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq, PartialOrd)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(indexed)]
    pub username: String,
    #[sea_orm(indexed)]
    pub email: Option<String>,
    pub password: Option<String>,
    pub domain: Option<String>,
    #[sea_orm(indexed, unique)]
    pub url: String,
    pub inbox_url: String,
    pub public_key: Option<String>,
    pub private_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Model {
    pub fn public_key(&self) -> Result<Option<RsaPublicKey>> {
        let Some(private_key) = self.private_key.as_ref() else {
            return Ok(None);
        };
        let private_key = RsaPrivateKey::from_pkcs8_pem(private_key)?;
        Ok(Some(private_key.to_public_key()))
    }
}

impl Zeroize for Model {
    fn zeroize(&mut self) {
        self.private_key.zeroize();
    }
}

impl Zeroize for ActiveModel {
    fn zeroize(&mut self) {
        self.private_key.take().zeroize();
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
