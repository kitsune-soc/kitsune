use crate::error::{Error, Result};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    schema::{accounts, posts, users},
    PgPool,
};
use smol_str::SmolStr;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct InstanceService {
    db_conn: PgPool,
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
        self.db_conn
            .with_connection(|mut db_conn| async move {
                accounts::table
                    .filter(accounts::local.eq(false))
                    .select(accounts::domain)
                    .distinct()
                    .count()
                    .get_result::<i64>(&mut db_conn)
                    .await
                    .map(|count| count as u64)
                    .map_err(Error::from)
            })
            .await
    }

    pub async fn local_post_count(&self) -> Result<u64> {
        self.db_conn
            .with_connection(|mut db_conn| async move {
                posts::table
                    .filter(posts::is_local.eq(true))
                    .count()
                    .get_result::<i64>(&mut db_conn)
                    .await
                    .map(|count| count as u64)
                    .map_err(Error::from)
            })
            .await
    }

    #[must_use]
    pub fn registrations_open(&self) -> bool {
        self.registrations_open
    }

    pub async fn user_count(&self) -> Result<u64> {
        self.db_conn
            .with_connection(|mut db_conn| async move {
                users::table
                    .count()
                    .get_result::<i64>(&mut db_conn)
                    .await
                    .map(|count| count as u64)
                    .map_err(Error::from)
            })
            .await
    }
}
