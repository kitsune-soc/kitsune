use crate::error::{Error, Result};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    schema::{accounts, posts, users},
    with_connection, PgPool,
};
use smol_str::SmolStr;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct InstanceService {
    db_pool: PgPool,
    #[builder(setter(into))]
    name: SmolStr,
    #[builder(setter(into))]
    description: SmolStr,
    character_limit: usize,
    registrations_open: bool,
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
        with_connection!(self.db_pool, |db_conn| {
            accounts::table
                .filter(accounts::local.eq(false))
                .select(accounts::domain)
                .distinct()
                .count()
                .get_result::<i64>(db_conn)
                .await
                .map(|count| count as u64)
        })
        .map_err(Error::from)
    }

    pub async fn local_post_count(&self) -> Result<u64> {
        with_connection!(self.db_pool, |db_conn| {
            posts::table
                .filter(posts::is_local.eq(true))
                .count()
                .get_result::<i64>(db_conn)
                .await
                .map(|count| count as u64)
        })
        .map_err(Error::from)
    }

    #[must_use]
    pub fn registrations_open(&self) -> bool {
        self.registrations_open
    }

    pub async fn user_count(&self) -> Result<u64> {
        with_connection!(self.db_pool, |db_conn| {
            users::table
                .count()
                .get_result::<i64>(db_conn)
                .await
                .map(|count| count as u64)
        })
        .map_err(Error::from)
    }
}
