use crate::error::{Error, Result};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    schema::{accounts, posts, users},
    PgPool,
};
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct InstanceService {
    db_conn: PgPool,
    #[builder(setter(into))]
    name: Arc<str>,
    #[builder(setter(into))]
    description: Arc<str>,
    character_limit: usize,
}

impl InstanceService {
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
        let mut db_conn = self.db_conn.get().await?;
        accounts::table
            .filter(accounts::local.eq(false))
            .select(accounts::domain)
            .group_by(accounts::domain)
            .count()
            .get_result::<i64>(&mut db_conn)
            .await
            .map(|count| count as u64)
            .map_err(Error::from)
    }

    pub async fn local_post_count(&self) -> Result<u64> {
        let mut db_conn = self.db_conn.get().await?;
        posts::table
            .filter(posts::is_local.eq(true))
            .count()
            .get_result::<i64>(&mut db_conn)
            .await
            .map(|count| count as u64)
            .map_err(Error::from)
    }

    pub async fn user_count(&self) -> Result<u64> {
        let mut db_conn = self.db_conn.get().await?;
        users::table
            .count()
            .get_result::<i64>(&mut db_conn)
            .await
            .map(|count| count as u64)
            .map_err(Error::from)
    }
}
