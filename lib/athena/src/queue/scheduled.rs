use crate::{error::Result, RedisPool};
use once_cell::sync::Lazy;
use rand::Rng;
use redis::Script;
use smol_str::SmolStr;
use std::{ops::RangeInclusive, time::Duration};
use typed_builder::TypedBuilder;

// The unit of this is seconds. The actor will wait between [START] and [END] seconds after executing the script.
// This is to prevent a bunch of blocking the server when multiple instances of this job server are running.
const SCHEDULE_PAUSE_RANGE: RangeInclusive<u64> = 5..=10;
// This functionality is expressed as a script since scripts are executed transactionally
static SCHEDULE_SCRIPT: Lazy<Script> =
    Lazy::new(|| Script::new(include_str!("../../lua/copy_scheduled.lua")));

#[derive(TypedBuilder)]
pub struct ScheduledJobActor {
    redis_pool: RedisPool,
    scheduled_queue_name: SmolStr,
    queue_name: SmolStr,
}

impl ScheduledJobActor {
    async fn run(&mut self) -> Result<()> {
        let mut conn = self.redis_pool.get();
        SCHEDULE_SCRIPT
            .key(self.queue_name.as_str())
            .key(self.scheduled_queue_name.as_str())
            .invoke_async(&mut conn)
            .await?;

        Ok(())
    }

    pub fn spawn(mut self) {
        tokio::spawn(async move {
            loop {
                if let Err(error) = self.run().await {
                    error!(?error, "cycle failed");
                }

                let sleep_duration =
                    Duration::from_secs(rand::thread_rng().gen_range(SCHEDULE_PAUSE_RANGE));
                tokio::time::sleep(sleep_duration).await;
            }
        });
    }
}
