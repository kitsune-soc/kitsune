use crate::error::{Error, Result};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::custom_emoji::CustomEmoji,
    schema::{accounts, custom_emojis, posts, users},
    PgPool,
};
use scoped_futures::ScopedFutureExt;
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
        self.db_pool
            .with_connection(|db_conn| {
                async move {
                    accounts::table
                        .filter(accounts::local.eq(false))
                        .select(accounts::domain)
                        .distinct()
                        .count()
                        .get_result::<i64>(db_conn)
                        .await
                        .map(|count| count as u64)
                }
                .scoped()
            })
            .await
            .map_err(Error::from)
    }

    pub async fn local_post_count(&self) -> Result<u64> {
        self.db_pool
            .with_connection(|db_conn| {
                async move {
                    posts::table
                        .filter(posts::is_local.eq(true))
                        .count()
                        .get_result::<i64>(db_conn)
                        .await
                        .map(|count| count as u64)
                }
                .scoped()
            })
            .await
            .map_err(Error::from)
    }

    #[must_use]
    pub fn registrations_open(&self) -> bool {
        self.registrations_open
    }

    pub async fn user_count(&self) -> Result<u64> {
        self.db_pool
            .with_connection(|db_conn| {
                async move {
                    users::table
                        .count()
                        .get_result::<i64>(db_conn)
                        .await
                        .map(|count| count as u64)
                }
                .scoped()
            })
            .await
            .map_err(Error::from)
    }
}
