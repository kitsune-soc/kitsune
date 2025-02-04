use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_config::instance::StatisticsMode;
use kitsune_db::{
    schema::{accounts, posts, users},
    with_connection, PgPool,
};
use kitsune_derive::kitsune_service;
use kitsune_error::{Error, Result};
use rand::seq::IteratorRandom;
use smol_str::SmolStr;
use std::{future::Future, ops::RangeInclusive};

const STATISTICS_RANGE: RangeInclusive<u64> = 24..=1312_1312;

#[inline]
fn random_statistic() -> u64 {
    STATISTICS_RANGE.choose(&mut rand::thread_rng()).unwrap()
}

#[kitsune_service]
pub struct InstanceService {
    db_pool: PgPool,
    #[builder(setter(into))]
    name: SmolStr,
    #[builder(setter(into))]
    description: SmolStr,
    character_limit: usize,
    registrations_open: bool,
    statistics_mode: StatisticsMode,
}

#[inline]
async fn with_statistics_mode<F, E>(mode: StatisticsMode, fut: F) -> Result<u64, E>
where
    F: Future<Output = Result<u64, E>>,
{
    match mode {
        StatisticsMode::Random => Ok(random_statistic()),
        StatisticsMode::Regular => fut.await,
        StatisticsMode::Zero => Ok(0),
    }
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
        with_statistics_mode(self.statistics_mode, async {
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
        })
        .await
    }

    pub async fn local_post_count(&self) -> Result<u64> {
        with_statistics_mode(self.statistics_mode, async {
            with_connection!(self.db_pool, |db_conn| {
                posts::table
                    .filter(posts::is_local.eq(true))
                    .count()
                    .get_result::<i64>(db_conn)
                    .await
                    .map(|count| count as u64)
            })
            .map_err(Error::from)
        })
        .await
    }

    #[must_use]
    pub fn registrations_open(&self) -> bool {
        self.registrations_open
    }

    pub async fn user_count(&self) -> Result<u64> {
        with_statistics_mode(self.statistics_mode, async {
            with_connection!(self.db_pool, |db_conn| {
                users::table
                    .count()
                    .get_result::<i64>(db_conn)
                    .await
                    .map(|count| count as u64)
            })
            .map_err(Error::from)
        })
        .await
    }
}
