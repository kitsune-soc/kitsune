use crate::{
    error::{Error, Result},
    util::generate_secret,
};
use chrono::Utc;
use derive_builder::Builder;
use kitsune_db::entity::oauth2_applications;
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel};
use uuid::Uuid;

#[derive(Builder, Clone)]
pub struct CreateApp {
    name: String,
    redirect_uris: String,
}

impl CreateApp {
    #[must_use]
    pub fn builder() -> CreateAppBuilder {
        CreateAppBuilder::default()
    }
}

#[derive(Builder, Clone)]
pub struct Oauth2Service {
    db_conn: DatabaseConnection,
}

impl Oauth2Service {
    #[must_use]
    pub fn builder() -> Oauth2ServiceBuilder {
        Oauth2ServiceBuilder::default()
    }

    pub async fn create_app(&self, create_app: CreateApp) -> Result<oauth2_applications::Model> {
        oauth2_applications::Model {
            id: Uuid::now_v7(),
            secret: generate_secret(),
            name: create_app.name,
            redirect_uri: create_app.redirect_uris,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }
        .into_active_model()
        .insert(&self.db_conn)
        .await
        .map_err(Error::from)
    }
}
