use self::{scheduled::ScheduledJobActor, util::StreamAutoClaimReply};
use crate::error::Result;
use deadpool_redis::Pool as RedisPool;
use either::Either;
use iso8601_timestamp::Timestamp;
use kitsune_uuid::Uuid;
use redis::{
    aio::ConnectionLike,
    streams::{StreamReadOptions, StreamReadReply},
    AsyncCommands, RedisResult,
};
use smol_str::SmolStr;
use std::{str::FromStr, time::Duration};
use tokio::sync::OnceCell;
use typed_builder::TypedBuilder;

mod scheduled;
mod util;

const CONSUMER_GROUP: &str = "kitsune-job-runners";
//const MIN_IDLE_TIME: Duration = Duration::from_secs(3600); // One hour should be enough to not overlap with job executions
const MIN_IDLE_TIME: Duration = Duration::from_secs(1);

#[derive(TypedBuilder)]
pub struct JobDetails {
    #[builder(default, setter(strip_option))]
    run_at: Option<Timestamp>,
}

#[derive(Clone, TypedBuilder)]
pub struct JobQueue {
    #[builder(default = Uuid::now_v7().to_string().into(), setter(into))]
    consumer_name: SmolStr,
    #[builder(setter(into))]
    queue_name: SmolStr,
    redis_pool: RedisPool,
    #[builder(
        default = SmolStr::from(format!("{queue_name}:scheduled")),
        setter(skip),
    )]
    scheduled_queue_name: SmolStr,

    #[builder(default, setter(skip))]
    group_initialised: OnceCell<()>,

    #[builder(
        default = ScheduledJobActor::builder()
            .queue_name(queue_name.clone())
            .redis_pool(redis_pool.clone())
            .scheduled_queue_name(scheduled_queue_name.clone())
            .build()
            .spawn(),
        setter(skip)
    )]
    _scheduled_actor: (),
}

impl JobQueue {
    async fn initialise_group<C>(&self, redis_conn: &mut C) -> Result<()>
    where
        C: ConnectionLike + Send + Sized,
    {
        self.group_initialised
            .get_or_try_init(|| async {
                let result: RedisResult<()> = redis_conn
                    .xgroup_create_mkstream(self.queue_name.as_str(), CONSUMER_GROUP, "0")
                    .await;

                if let Err(err) = result {
                    if err.kind() != redis::ErrorKind::ExtensionError {
                        return Err(err);
                    }
                }

                Ok(())
            })
            .await?;

        Ok(())
    }

    pub async fn enqueue(&self, job_details: JobDetails) -> Result<()> {
        let mut redis_conn = self.redis_pool.get().await?;
        let job_id = Uuid::now_v7();

        if let Some(run_at) = job_details.run_at {
            let score = run_at.duration_since(Timestamp::UNIX_EPOCH).whole_seconds();
            redis_conn
                .zadd(
                    self.scheduled_queue_name.as_str(),
                    job_id.to_string(),
                    score,
                )
                .await?;
        } else {
            redis_conn
                .xadd(
                    self.queue_name.as_str(),
                    "*",
                    &[("job_id", job_id.to_string())],
                )
                .await?;
        }

        todo!("Add payload into database");

        Ok(())
    }

    async fn fetch_job_ids(&self, max_jobs: usize) -> Result<impl Iterator<Item = Uuid>> {
        let mut redis_conn = self.redis_pool.get().await?;
        self.initialise_group(&mut redis_conn).await?;

        let StreamAutoClaimReply { claimed_ids, .. }: StreamAutoClaimReply =
            redis::cmd("XAUTOCLAIM")
                .arg(self.queue_name.as_str())
                .arg(CONSUMER_GROUP)
                .arg(self.consumer_name.as_str())
                .arg(MIN_IDLE_TIME.as_millis() as u64)
                .arg("0-0")
                .arg("COUNT")
                .arg(max_jobs)
                .query_async(&mut redis_conn)
                .await?;

        let claimed_ids = if claimed_ids.len() == max_jobs {
            Either::Left(claimed_ids.into_iter())
        } else {
            // TODO: Block for only ~2sec in case we claimed some jobs with the previous command
            let read_opts = StreamReadOptions::default()
                .count(max_jobs - claimed_ids.len())
                .group(CONSUMER_GROUP, self.consumer_name.as_str());

            let StreamReadReply { keys }: StreamReadReply = redis_conn
                .xread_options(&[self.queue_name.as_str()], &[">"], &read_opts)
                .await?;

            Either::Right(
                claimed_ids
                    .into_iter()
                    .chain(keys.into_iter().flat_map(|key| key.ids)),
            )
        };

        let id_iterator = claimed_ids.map(|id| {
            let job_id: String =
                redis::from_redis_value(&id.map["job_id"]).expect("[Bug] Malformed Job ID");
            Uuid::from_str(&job_id).expect("[Bug] Job ID is not a UUID")
        });

        Ok(id_iterator)
    }
}
