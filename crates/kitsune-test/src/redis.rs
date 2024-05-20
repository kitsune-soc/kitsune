use fred::{
    clients::RedisClient,
    interfaces::{ClientLike, KeysInterface, RedisResult, ServerInterface},
    types::{PerformanceConfig, RedisConfig, RedisValue, SetOptions},
};
use rand::Rng;
use std::{ops::RangeInclusive, time::Duration};

const DATABASE_RANGE: RangeInclusive<u8> = 1..=15;
const LOCK_KEY: &str = "_TEST_LOCK";
const LOCK_VALUE: &str = "LOCKED";
const SLEEP_DURATION: Duration = Duration::from_millis(100);

async fn switch_and_try_lock(conn: &RedisClient, id: u8) -> bool {
    conn.select(id).await.unwrap();
    try_lock(conn).await
}

async fn try_lock(conn: &RedisClient) -> bool {
    let Ok(value): RedisResult<RedisValue> = conn
        .set(LOCK_KEY, LOCK_VALUE, None, Some(SetOptions::NX), false)
        .await
    else {
        return false;
    };

    value.is_ok()
}

/// Find and claim one of the 16 database slots on the Redis instance
pub async fn find_unused_database(config: &RedisConfig) -> u8 {
    let connection = RedisClient::new(
        config.clone(),
        Some(PerformanceConfig {
            auto_pipeline: false,
            ..PerformanceConfig::default()
        }),
        None,
        None,
    );
    connection.init().await.unwrap();

    for i in DATABASE_RANGE {
        if switch_and_try_lock(&connection, i).await {
            return i;
        }
    }

    loop {
        let db_id = rand::thread_rng().gen_range(DATABASE_RANGE);
        if switch_and_try_lock(&connection, db_id).await {
            break db_id;
        }

        tokio::time::sleep(SLEEP_DURATION).await;
    }
}
