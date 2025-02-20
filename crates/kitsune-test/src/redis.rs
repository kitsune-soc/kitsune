use fred::{
    clients::Client,
    error::Error,
    interfaces::{ClientLike, KeysInterface, ServerInterface},
    types::{SetOptions, Value, config::Config},
};
use rand::Rng;
use std::{ops::RangeInclusive, time::Duration};

const DATABASE_RANGE: RangeInclusive<u8> = 1..=15;
const LOCK_KEY: &str = "_TEST_LOCK";
const LOCK_VALUE: &str = "LOCKED";
const SLEEP_DURATION: Duration = Duration::from_millis(100);

async fn switch_and_try_lock(conn: &Client, id: u8) -> bool {
    conn.select(Value::Integer(id.into())).await.unwrap();
    try_lock(conn).await
}

async fn try_lock(conn: &Client) -> bool {
    let Ok(value): Result<Value, Error> = conn
        .set(LOCK_KEY, LOCK_VALUE, None, Some(SetOptions::NX), false)
        .await
    else {
        return false;
    };

    value.is_ok()
}

/// Find and claim one of the 16 database slots on the Redis instance
pub async fn find_unused_database(config: &Config) -> u8 {
    let connection = Client::new(config.clone(), None, None, None);
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
