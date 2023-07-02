use self::{scheduled::ScheduledJobActor, util::StreamAutoClaimReply};
use crate::{error::Result, impl_to_redis_args, Error, JobContextRepository, Runnable};
use ahash::AHashMap;
use deadpool_redis::Pool as RedisPool;
use either::Either;
use exponential_backoff::Backoff;
use futures_util::StreamExt;
use iso8601_timestamp::Timestamp;
use rand::Rng;
use redis::{
    aio::ConnectionLike,
    streams::{StreamReadOptions, StreamReadReply},
    AsyncCommands, RedisResult,
};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use speedy_uuid::Uuid;
use std::{
    fmt::Debug, future::Future, ops::RangeInclusive, str::FromStr, sync::Arc, time::Duration,
};
use tokio::{sync::OnceCell, task::JoinSet, time::Instant};
use typed_builder::TypedBuilder;

mod scheduled;
mod util;

const BLOCK_TIME: Duration = Duration::from_secs(2);
const ERROR_SLEEP_RANGE_SECS: RangeInclusive<u64> = 3..=6;
const MIN_IDLE_TIME: Duration = Duration::from_secs(10 * 60);

const MAX_RETRIES: u32 = 10;
const MIN_BACKOFF_DURATION: Duration = Duration::from_secs(5);

async fn rerun_until_success<F, Fut, Ok, Err>(func: F) -> Ok
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Ok, Err>>,
    Err: Debug,
{
    loop {
        match func().await {
            Ok(val) => break val,
            Err(error) => {
                error!(?error, "job completion routine failed");

                let sleep_duration =
                    Duration::from_secs(rand::thread_rng().gen_range(ERROR_SLEEP_RANGE_SECS));
                tokio::time::sleep(sleep_duration).await;
            }
        }
    }
}

enum JobState<'a> {
    Succeeded {
        job_id: Uuid,
        stream_id: &'a str,
    },
    Failed {
        fail_count: u32,
        job_id: Uuid,
        stream_id: &'a str,
    },
}

impl JobState<'_> {
    fn stream_id(&self) -> &str {
        match self {
            Self::Succeeded { stream_id, .. } | Self::Failed { stream_id, .. } => stream_id,
        }
    }
}

#[derive(TypedBuilder)]
pub struct JobDetails<C> {
    context: C,
    #[builder(default)]
    fail_count: u32,
    #[builder(default = Uuid::now_v7())]
    job_id: Uuid,
    #[builder(default, setter(strip_option))]
    run_at: Option<Timestamp>,
}

struct JobData {
    stream_id: String,
    meta: JobMeta,
}

impl_to_redis_args! {
    #[derive(Deserialize, Serialize)]
    struct JobMeta {
        job_id: Uuid,
        fail_count: u32,
    }
}

#[derive(TypedBuilder)]
pub struct JobQueue<CR> {
    #[builder(default = "kitsune-job-runners".into(), setter(into))]
    consumer_group: SmolStr,
    #[builder(default = Uuid::now_v7().to_string().into(), setter(into))]
    consumer_name: SmolStr,
    #[builder(setter(into))]
    context_repository: Arc<CR>,
    #[builder(setter(into))]
    queue_name: SmolStr,
    redis_pool: RedisPool,
    #[builder(default = SmolStr::from(format!("{queue_name}:scheduled")))]
    scheduled_queue_name: SmolStr,

    #[builder(default, setter(skip))]
    group_initialised: Arc<OnceCell<()>>,

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

impl<CR> JobQueue<CR>
where
    CR: JobContextRepository + Send + Sync + 'static,
{
    async fn initialise_group<C>(&self, redis_conn: &mut C) -> Result<()>
    where
        C: ConnectionLike + Send + Sized,
    {
        self.group_initialised
            .get_or_try_init(|| async {
                let result: RedisResult<()> = redis_conn
                    .xgroup_create_mkstream(
                        self.queue_name.as_str(),
                        self.consumer_group.as_str(),
                        "0",
                    )
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

    fn enqueue_redis_cmd(
        &self,
        job_meta: &JobMeta,
        run_at: Option<Timestamp>,
    ) -> Result<redis::Cmd> {
        let cmd = if let Some(run_at) = run_at {
            let score = run_at.duration_since(Timestamp::UNIX_EPOCH).whole_seconds();
            redis::Cmd::zadd(
                self.scheduled_queue_name.as_str(),
                simd_json::to_string(job_meta)?,
                score,
            )
        } else {
            let mut cmd = redis::cmd("XADD");
            cmd.arg(self.queue_name.as_str()).arg("*").arg(job_meta);
            cmd
        };

        Ok(cmd)
    }

    pub async fn enqueue(&self, job_details: JobDetails<CR::JobContext>) -> Result<()> {
        let job_meta = JobMeta {
            job_id: job_details.job_id,
            fail_count: job_details.fail_count,
        };

        self.context_repository
            .store_context(job_meta.job_id, job_details.context)
            .await
            .map_err(|err| Error::ContextRepository(err.into()))?;

        let mut redis_conn = self.redis_pool.get().await?;
        self.enqueue_redis_cmd(&job_meta, job_details.run_at)?
            .query_async(&mut redis_conn)
            .await?;

        Ok(())
    }

    async fn fetch_job_data(
        &self,
        max_jobs: usize,
    ) -> Result<impl Iterator<Item = JobData> + Clone> {
        let mut redis_conn = self.redis_pool.get().await?;
        self.initialise_group(&mut redis_conn).await?;

        let StreamAutoClaimReply { claimed_ids, .. }: StreamAutoClaimReply =
            redis::cmd("XAUTOCLAIM")
                .arg(self.queue_name.as_str())
                .arg(self.consumer_group.as_str())
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
            let mut read_opts = StreamReadOptions::default()
                .count(max_jobs - claimed_ids.len())
                .group(self.consumer_group.as_str(), self.consumer_name.as_str());

            read_opts = if claimed_ids.is_empty() {
                read_opts.block(0)
            } else {
                read_opts.block(BLOCK_TIME.as_millis() as usize)
            };

            let read_reply: Option<StreamReadReply> = redis_conn
                .xread_options(&[self.queue_name.as_str()], &[">"], &read_opts)
                .await?;

            let read_ids = read_reply
                .into_iter()
                .flat_map(|reply| reply.keys.into_iter().flat_map(|key| key.ids));

            Either::Right(claimed_ids.into_iter().chain(read_ids))
        };

        let job_data_iterator = claimed_ids.map(|id| {
            let job_id: String =
                redis::from_redis_value(&id.map["job_id"]).expect("[Bug] Malformed Job ID");
            let job_id = Uuid::from_str(&job_id).expect("[Bug] Job ID is not a UUID");
            let fail_count: u32 =
                redis::from_redis_value(&id.map["fail_count"]).expect("[Bug] Malformed fail count");

            JobData {
                stream_id: id.id,
                meta: JobMeta { job_id, fail_count },
            }
        });

        Ok(job_data_iterator)
    }

    async fn complete_job(&self, state: &JobState<'_>) -> Result<()> {
        let mut pipeline = redis::pipe();
        pipeline
            .atomic()
            .ignore()
            .xack(
                self.queue_name.as_str(),
                self.consumer_group.as_str(),
                &[state.stream_id()],
            )
            .xdel(self.queue_name.as_str(), &[state.stream_id()]);

        if let JobState::Failed {
            fail_count, job_id, ..
        } = state
        {
            let backoff = Backoff::new(MAX_RETRIES, MIN_BACKOFF_DURATION, None);
            if let Some(backoff_duration) = backoff.next(*fail_count) {
                let job_meta = JobMeta {
                    job_id: *job_id,
                    fail_count: fail_count + 1,
                };
                let backoff_timestamp = Timestamp::now_utc() + backoff_duration;
                let enqueue_cmd = self.enqueue_redis_cmd(&job_meta, Some(backoff_timestamp))?;

                pipeline.add_command(enqueue_cmd);
            }
        }

        {
            let mut conn = self.redis_pool.get().await?;
            pipeline.query_async::<_, ()>(&mut conn).await?;
        }

        if let JobState::Succeeded { job_id, .. } = state {
            self.context_repository
                .remove_context(*job_id)
                .await
                .map_err(|err| Error::ContextRepository(err.into()))?;
        }

        Ok(())
    }

    async fn reclaim_job(&self, job_data: &JobData) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        conn.xclaim(
            self.queue_name.as_str(),
            self.consumer_group.as_str(),
            self.consumer_name.as_str(),
            0,
            &[job_data.stream_id.as_str()],
        )
        .await?;

        Ok(())
    }

    pub async fn spawn_jobs(
        &self,
        max_jobs: usize,
        run_ctx: Arc<<CR::JobContext as Runnable>::Context>,
        existing_joinset: Option<JoinSet<()>>,
    ) -> Result<JoinSet<()>> {
        let job_data = self.fetch_job_data(max_jobs).await?;
        let context_stream = self
            .context_repository
            .fetch_context(job_data.clone().map(|data| data.meta.job_id))
            .await
            .map_err(|err| Error::ContextRepository(err.into()))?;

        tokio::pin!(context_stream);

        // Collect all the job data into a hashmap indexed by the job ID
        // This is because we don't enforce an ordering with the batch fetching
        let job_data = job_data
            .map(|data| (data.meta.job_id, data))
            .collect::<AHashMap<Uuid, JobData>>();
        let job_data = Arc::new(job_data);

        let mut join_set = existing_joinset.unwrap_or_else(JoinSet::new);
        while let Some((job_id, job_ctx)) = context_stream
            .next()
            .await
            .transpose()
            .map_err(|err| Error::ContextRepository(err.into()))?
        {
            let this = self.clone();
            let job_data = Arc::clone(&job_data);
            let run_ctx = Arc::clone(&run_ctx);

            join_set.spawn(async move {
                let job_data = &job_data[&job_id];
                let mut run_fut = job_ctx.run(&run_ctx);

                let tick_period = MIN_IDLE_TIME - Duration::from_secs(2 * 60);
                let mut tick_interval =
                    tokio::time::interval_at(Instant::now() + tick_period, tick_period);

                let result = loop {
                    tokio::select! {
                        result = &mut run_fut => break result,
                        _ = tick_interval.tick() => {
                            rerun_until_success(|| this.reclaim_job(job_data)).await;
                        }
                    }
                };

                let job_state = if let Err(error) = result {
                    error!(error = ?error.into(), "Failed run job");
                    JobState::Failed {
                        fail_count: job_data.meta.fail_count,
                        job_id,
                        stream_id: &job_data.stream_id,
                    }
                } else {
                    JobState::Succeeded {
                        job_id,
                        stream_id: &job_data.stream_id,
                    }
                };

                rerun_until_success(|| this.complete_job(&job_state)).await;
            });
        }

        Ok(join_set)
    }
}

impl<CR> Clone for JobQueue<CR> {
    fn clone(&self) -> Self {
        Self {
            consumer_group: self.consumer_group.clone(),
            consumer_name: self.consumer_name.clone(),
            context_repository: self.context_repository.clone(),
            queue_name: self.queue_name.clone(),
            redis_pool: self.redis_pool.clone(),
            scheduled_queue_name: self.scheduled_queue_name.clone(),
            group_initialised: self.group_initialised.clone(),
            _scheduled_actor: (),
        }
    }
}
