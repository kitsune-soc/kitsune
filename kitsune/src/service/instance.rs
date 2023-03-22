use crate::error::{Error, Result};
use derive_builder::Builder;
use kitsune_db::entity::{
    accounts, posts,
    prelude::{Accounts, Posts, Users},
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
};
use std::sync::Arc;

#[derive(Builder, Clone)]
pub struct InstanceService {
    db_conn: DatabaseConnection,
    #[builder(setter(into))]
    name: Arc<str>,
    #[builder(setter(into))]
    description: Arc<str>,
    character_limit: usize,
}

impl InstanceService {
    #[must_use]
    pub fn builder() -> InstanceServiceBuilder {
        InstanceServiceBuilder::default()
    }

    #[must_use]
    pub fn character_limit(&self) -> usize {
        self.character_limit
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn known_instances(&self) -> Result<u64> {
        Accounts::find()
            .filter(accounts::Column::Local.eq(false))
            .select_only()
            .group_by(accounts::Column::Domain)
            .count(&self.db_conn)
            .await
            .map_err(Error::from)
    }

    pub async fn local_post_count(&self) -> Result<u64> {
        Posts::find()
            .filter(posts::Column::IsLocal.eq(true))
            .count(&self.db_conn)
            .await
            .map_err(Error::from)
    }

    pub async fn user_count(&self) -> Result<u64> {
        Users::find()
            .count(&self.db_conn)
            .await
            .map_err(Error::from)
    }
}
