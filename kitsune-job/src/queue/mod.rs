use self::{scheduled::ScheduledJobActor, util::StreamAutoClaimReply};
use crate::{error::Result, impl_to_redis_args, Error, JobContextRepository, Runnable};
use ahash::AHashMap;
use deadpool_redis::Pool as RedisPool;
use either::Either;
use exponential_backoff::Backoff;
use futures_util::StreamExt;
use iso8601_timestamp::Timestamp;
use kitsune_uuid::Uuid;
use redis::{
    aio::ConnectionLike,
    streams::{StreamReadOptions, StreamReadReply},
    AsyncCommands, RedisResult,
};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::{str::FromStr, sync::Arc, time::Duration};
use tokio::{sync::OnceCell, task::JoinSet};
use typed_builder::TypedBuilder;

mod scheduled;
mod util;

const BLOCK_TIME: Duration = Duration::from_secs(2);
const CONSUMER_GROUP: &str = "kitsune-job-runners";
//const MIN_IDLE_TIME: Duration = Duration::from_secs(3600); // One hour should be enough to not overlap with job executions
const MIN_IDLE_TIME: Duration = Duration::from_secs(1);

const MAX_RETRIES: u32 = 10;
const MIN_BACKOFF_DURATION: Duration = Duration::from_secs(5);

enum JobState<C> {
    Succeeded {
        job_id: Uuid,
    },
    Failed {
        context: C,
        fail_count: u32,
        job_id: Uuid,
    },
}

impl<C> JobState<C> {
    fn job_id(&self) -> Uuid {
        match self {
            Self::Succeeded { job_id } | Self::Failed { job_id, .. } => *job_id,
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

impl_to_redis_args! {
    #[derive(Deserialize, Serialize)]
    struct JobMeta {
        job_id: Uuid,
        fail_count: u32,
    }
}

#[derive(TypedBuilder)]
pub struct JobQueue<CR> {
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

    pub async fn enqueue(&self, job_details: JobDetails<CR::JobContext>) -> Result<()> {
        let mut redis_conn = self.redis_pool.get().await?;
        let job_meta = JobMeta {
            job_id: job_details.job_id,
            fail_count: job_details.fail_count,
        };

        if let Some(run_at) = job_details.run_at {
            let score = run_at.duration_since(Timestamp::UNIX_EPOCH).whole_seconds();
            redis_conn
                .zadd(
                    self.scheduled_queue_name.as_str(),
                    simd_json::to_string(&job_meta)?,
                    score,
                )
                .await?;
        } else {
            redis::cmd("XADD")
                .arg(self.queue_name.as_str())
                .arg("*")
                .arg(&job_meta)
                .query_async(&mut redis_conn)
                .await?;
        }

        self.context_repository
            .store_context(job_meta.job_id, job_details.context)
            .await
            .map_err(|err| Error::ContextRepository(err.into()))?;

        Ok(())
    }

    async fn fetch_job_ids(
        &self,
        max_jobs: usize,
    ) -> Result<impl Iterator<Item = JobMeta> + Clone> {
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
            let mut read_opts = StreamReadOptions::default()
                .count(max_jobs - claimed_ids.len())
                .group(CONSUMER_GROUP, self.consumer_name.as_str());

            if !claimed_ids.is_empty() {
                read_opts = read_opts.block(BLOCK_TIME.as_millis() as usize);
            }

            let read_reply: Option<StreamReadReply> = redis_conn
                .xread_options(&[self.queue_name.as_str()], &[">"], &read_opts)
                .await?;

            let read_ids = read_reply
                .into_iter()
                .flat_map(|reply| reply.keys.into_iter().flat_map(|key| key.ids));

            Either::Right(claimed_ids.into_iter().chain(read_ids))
        };

        let job_meta_iterator = claimed_ids.map(|id| {
            let job_id: String =
                redis::from_redis_value(&id.map["job_id"]).expect("[Bug] Malformed Job ID");
            let job_id = Uuid::from_str(&job_id).expect("[Bug] Job ID is not a UUID");
            let fail_count: u32 =
                redis::from_redis_value(&id.map["fail_count"]).expect("[Bug] Malformed fail count");

            JobMeta { job_id, fail_count }
        });

        Ok(job_meta_iterator)
    }

    async fn complete_job(&self, state: JobState<CR::JobContext>) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        redis::pipe()
            .atomic()
            .xack(self.queue_name.as_str(), CONSUMER_GROUP, &[state.job_id()])
            .xdel(self.queue_name.as_str(), &[state.job_id()])
            .query_async(&mut conn)
            .await?;

        self.context_repository
            .remove_context(state.job_id())
            .await
            .map_err(|err| Error::ContextRepository(err.into()))?;

        if let JobState::Failed {
            context,
            fail_count,
            job_id,
        } = state
        {
            let backoff = Backoff::new(MAX_RETRIES, MIN_BACKOFF_DURATION, None);
            if let Some(backoff_duration) = backoff.next(fail_count) {
                self.enqueue(
                    JobDetails::builder()
                        .context(context)
                        .fail_count(fail_count + 1)
                        .job_id(job_id)
                        .run_at(Timestamp::now_utc() + backoff_duration)
                        .build(),
                )
                .await?;
            }
        }

        todo!()
    }

    pub async fn spawn_jobs(
        &self,
        max_jobs: usize,
        run_ctx: Arc<<CR::JobContext as Runnable>::Context>,
    ) -> Result<JoinSet<()>> {
        let job_meta = self.fetch_job_ids(max_jobs).await?;
        let context_stream = self
            .context_repository
            .fetch_context(job_meta.clone().map(|meta| meta.job_id))
            .await
            .map_err(|err| Error::ContextRepository(err.into()))?;

        tokio::pin!(context_stream);

        // Collect all the job metadata into a hashmap indexed by the job ID
        // This is because we don't enforce an ordering with the batch fetching
        let job_meta = job_meta
            .map(|meta| (meta.job_id, meta))
            .collect::<AHashMap<Uuid, JobMeta>>();
        let job_meta = Arc::new(job_meta);

        let mut join_set = JoinSet::new();
        while let Some((job_id, job_ctx)) = context_stream
            .next()
            .await
            .transpose()
            .map_err(|err| Error::ContextRepository(err.into()))?
        {
            let this = self.clone();
            let job_meta = Arc::clone(&job_meta);
            let run_ctx = Arc::clone(&run_ctx);

            join_set.spawn(async move {
                let job_state = if let Err(ref error) = job_ctx.run(&run_ctx).await {
                    error!(?error, "Failed run job");

                    let job_meta = &job_meta[&job_id];
                    JobState::Failed {
                        context: job_ctx,
                        fail_count: job_meta.fail_count,
                        job_id,
                    }
                } else {
                    JobState::Succeeded { job_id }
                };

                let _ = this.complete_job(job_state).await;
            });
        }

        Ok(join_set)
    }
}

impl<CR> Clone for JobQueue<CR> {
    fn clone(&self) -> Self {
        Self {
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
