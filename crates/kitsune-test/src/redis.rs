use rand::Rng;
use redis::{aio::MultiplexedConnection, RedisResult, Value};
use std::{ops::RangeInclusive, time::Duration};

const DATABASE_RANGE: RangeInclusive<u8> = 1..=15;
const LOCK_KEY: &str = "_TEST_LOCK";
const LOCK_VALUE: &str = "LOCKED";
const SLEEP_DURATION: Duration = Duration::from_millis(100);

async fn switch_and_try_lock(conn: &mut MultiplexedConnection, id: u8) -> bool {
    let (): () = redis::cmd("SELECT")
        .arg(id)
        .query_async(conn)
        .await
        .unwrap();

    try_lock(conn).await
}

async fn try_lock(conn: &mut MultiplexedConnection) -> bool {
    let result: RedisResult<Value> = redis::cmd("SET")
        .arg(LOCK_KEY)
        .arg(LOCK_VALUE)
        .arg("NX")
        .query_async(conn)
        .await;

    matches!(result, Ok(Value::Okay))
}

/// Find and claim one of the 16 database slots on the Redis instance
pub async fn find_unused_database(client: &redis::Client) -> u8 {
    let mut connection = client.get_multiplexed_async_connection().await.unwrap();

    for i in DATABASE_RANGE {
        if switch_and_try_lock(&mut connection, i).await {
            return i;
        }
    }

    loop {
        let db_id = rand::thread_rng().gen_range(DATABASE_RANGE);
        if switch_and_try_lock(&mut connection, db_id).await {
            break db_id;
        }

        tokio::time::sleep(SLEEP_DURATION).await;
    }
}
