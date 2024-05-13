use self::scheduled::ScheduledJobActor;
use crate::{
    consts::{BLOCK_TIME, MAX_RETRIES, MIN_IDLE_TIME},
    error::Result,
    Error, JobContextRepository, JobData, JobDetails, JobResult, KeeperOfTheSecrets, Outcome,
};
use async_trait::async_trait;
use either::Either;
use fred::{
    clients::RedisPool,
    interfaces::{SortedSetsInterface, StreamsInterface},
    types::{RedisValue, XID},
};
use iso8601_timestamp::Timestamp;
use just_retry::{
    retry_policies::{policies::ExponentialBackoff, Jitter},
    JustRetryPolicy, StartTime,
};
use smol_str::SmolStr;
use speedy_uuid::Uuid;
use std::{ops::ControlFlow, str::FromStr, time::SystemTime};
use tokio::sync::OnceCell;
use triomphe::Arc;
use typed_builder::TypedBuilder;

mod scheduled;

#[derive(TypedBuilder)]
pub struct JobQueue<CR> {
    #[builder(default = "athena-job-runners".into(), setter(into))]
    consumer_group: SmolStr,
    #[builder(default = Uuid::now_v7().to_string().into(), setter(into))]
    consumer_name: SmolStr,
    #[builder(setter(into))]
    context_repository: Arc<CR>,
    #[builder(default = MAX_RETRIES)]
    max_retries: u32,
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
    async fn initialise_group(&self) -> Result<()> {
        self.group_initialised
            .get_or_try_init(|| {
                self.redis_pool.xgroup_create(
                    self.queue_name.as_str(),
                    self.consumer_group.as_str(),
                    "0",
                    true,
                )
            })
            .await?;

        Ok(())
    }

    async fn enqueue_ops<C>(
        &self,
        client: &C,
        job_meta: &JobData,
        run_at: Option<Timestamp>,
    ) -> Result<()>
    where
        C: SortedSetsInterface + StreamsInterface,
    {
        let cmd = if let Some(run_at) = run_at {
            let score = run_at.duration_since(Timestamp::UNIX_EPOCH).whole_seconds();
            client
                .zadd(
                    self.scheduled_queue_name.as_str(),
                    None,
                    None,
                    true,
                    false,
                    (score as f64, simd_json::to_string(job_meta)?),
                )
                .await?;
        } else {
            client
                .xadd(
                    self.queue_name.as_str(),
                    true,
                    None,
                    XID::Auto,
                    vec![
                        ("job_id", RedisValue::from(job_meta.job_id)),
                        ("fail_count", RedisValue::from(job_meta.fail_count)),
                    ],
                )
                .await?;
        };

        Ok(cmd)
    }
}

#[async_trait]
impl<CR> crate::JobQueue for JobQueue<CR>
where
    CR: JobContextRepository + Send + Sync + 'static,
{
    type ContextRepository = CR;

    #[inline]
    fn context_repository(&self) -> &Self::ContextRepository {
        &self.context_repository
    }

    async fn enqueue(&self, job_details: JobDetails<CR::JobContext>) -> Result<()> {
        let job_data = JobData {
            job_id: job_details.job_id,
            fail_count: job_details.fail_count,
            ctx: KeeperOfTheSecrets::empty(),
        };

        self.context_repository
            .store_context(job_data.job_id, job_details.context)
            .await
            .map_err(|err| Error::ContextRepository(err.into()))?;

        self.enqueue_ops(&self.redis_pool, &job_data, job_details.run_at)
            .await?;

        Ok(())
    }

    async fn fetch_job_data(&self, max_jobs: usize) -> Result<Vec<JobData>> {
        self.initialise_group().await?;

        let (_start, claimed_ids) = self
            .redis_pool
            .xautoclaim_values(
                self.queue_name.as_str(),
                self.consumer_group.as_str(),
                self.consumer_name.as_str(),
                MIN_IDLE_TIME.as_millis() as u64,
                "0-0",
                Some(max_jobs as u64),
                false,
            )
            .await?;

        let claimed_ids = if claimed_ids.len() == max_jobs {
            Either::Left(claimed_ids.into_iter())
        } else {
            let block_time = if claimed_ids.is_empty() {
                0
            } else {
                BLOCK_TIME.as_millis()
            };

            let read_reply = self
                .redis_pool
                .xreadgroup_map(
                    self.consumer_group.as_str(),
                    self.consumer_name.as_str(),
                    Some((max_jobs - claimed_ids.len()) as u64),
                    Some(block_time as u64),
                    false,
                    self.queue_name.as_str(),
                    XID::NewInGroup,
                )
                .await?;

            let read_ids = read_reply
                .into_iter()
                .flat_map(|reply| reply.keys.into_iter().flat_map(|key| key.ids));

            Either::Right(claimed_ids.into_iter().chain(read_ids))
        };

        let job_data = claimed_ids
            .map(|id| {
                let job_id: String =
                    redis::from_redis_value(&id.map["job_id"]).expect("[Bug] Malformed Job ID");
                let job_id = Uuid::from_str(&job_id).expect("[Bug] Job ID is not a UUID");
                let fail_count: u32 = redis::from_redis_value(&id.map["fail_count"])
                    .expect("[Bug] Malformed fail count");

                JobData {
                    ctx: KeeperOfTheSecrets::new(id.id),
                    job_id,
                    fail_count,
                }
            })
            .collect();

        Ok(job_data)
    }

    async fn complete_job(&self, state: &JobResult<'_>) -> Result<()> {
        let stream_id = state
            .ctx
            .get::<String>()
            .expect("[Bug] Not a string in the context");

        let client = self.redis_pool.next();
        let pipeline = client.pipeline();

        pipeline
            .xack(
                self.queue_name.as_str(),
                self.consumer_group.as_str(),
                stream_id.as_str(),
            )
            .await?;
        pipeline
            .xdel(self.queue_name.as_str(), &[stream_id])
            .await?;

        let remove_context = match state.outcome {
            Outcome::Fail { fail_count } => {
                let backoff = ExponentialBackoff::builder()
                    .jitter(Jitter::Bounded)
                    .build_with_max_retries(self.max_retries);

                if let ControlFlow::Continue(delta) =
                    backoff.should_retry(StartTime::Irrelevant, fail_count)
                {
                    let job_data = JobData {
                        job_id: state.job_id,
                        fail_count: fail_count + 1,
                        ctx: KeeperOfTheSecrets::empty(),
                    };

                    let backoff_timestamp = Timestamp::from(SystemTime::now() + delta);
                    self.enqueue_ops(&pipeline, &job_data, Some(backoff_timestamp))
                        .await?;

                    false // Do not delete the context if we were able to retry the job one more time
                } else {
                    true // We hit the maximum amount of retries, we won't re-enqueue the job, so we can just remove the context
                }
            }
            Outcome::Success => true, // Execution succeeded, we don't need the context anymore
        };

        pipeline.last().await?;

        if remove_context {
            self.context_repository
                .remove_context(state.job_id)
                .await
                .map_err(|err| Error::ContextRepository(err.into()))?;
        }

        Ok(())
    }

    async fn reclaim_job(&self, job_data: &JobData) -> Result<()> {
        let stream_id = job_data
            .ctx
            .get::<String>()
            .expect("[Bug] Not a string in the context");

        self.redis_pool
            .xclaim(
                self.queue_name.as_str(),
                self.consumer_group.as_str(),
                self.consumer_name.as_str(),
                0,
                stream_id.as_str(),
                None,
                None,
                None,
                true,
                false,
            )
            .await?;

        Ok(())
    }
}

impl<CR> Clone for JobQueue<CR> {
    fn clone(&self) -> Self {
        Self {
            consumer_group: self.consumer_group.clone(),
            consumer_name: self.consumer_name.clone(),
            context_repository: self.context_repository.clone(),
            max_retries: self.max_retries,
            queue_name: self.queue_name.clone(),
            redis_pool: self.redis_pool.clone(),
            scheduled_queue_name: self.scheduled_queue_name.clone(),
            group_initialised: self.group_initialised.clone(),
            _scheduled_actor: (),
        }
    }
}
