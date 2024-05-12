use self::scheduled::ScheduledJobActor;
use crate::{
    consts::{BLOCK_TIME, MAX_RETRIES, MIN_IDLE_TIME},
    error::Result,
    Error, JobContextRepository, JobData, JobDetails, JobResult, KeeperOfTheSecrets, Outcome,
};
use async_trait::async_trait;
use either::Either;
use iso8601_timestamp::Timestamp;
use just_retry::{
    retry_policies::{policies::ExponentialBackoff, Jitter},
    JustRetryPolicy, StartTime,
};
use redis::{
    aio::ConnectionLike,
    streams::{StreamAutoClaimOptions, StreamAutoClaimReply, StreamReadOptions, StreamReadReply},
    AsyncCommands, RedisResult,
};
use smol_str::SmolStr;
use speedy_uuid::Uuid;
use std::{ops::ControlFlow, str::FromStr, time::SystemTime};
use tokio::sync::OnceCell;
use triomphe::Arc;
use typed_builder::TypedBuilder;

mod scheduled;

type Pool = multiplex_pool::Pool<redis::aio::ConnectionManager>;

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
    redis_pool: Pool,
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
        job_meta: &JobData,
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
            cmd.arg(self.queue_name.as_str())
                .arg("*")
                .arg("job_id")
                .arg(job_meta.job_id)
                .arg("fail_count")
                .arg(job_meta.fail_count);

            cmd
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

        let mut redis_conn = self.redis_pool.get();
        self.enqueue_redis_cmd(&job_data, job_details.run_at)?
            .query_async(&mut redis_conn)
            .await?;

        Ok(())
    }

    async fn fetch_job_data(&self, max_jobs: usize) -> Result<Vec<JobData>> {
        let mut redis_conn = self.redis_pool.get();
        self.initialise_group(&mut redis_conn).await?;

        let StreamAutoClaimReply {
            claimed: claimed_ids,
            ..
        } = redis_conn
            .xautoclaim_options(
                self.queue_name.as_str(),
                self.consumer_group.as_str(),
                self.consumer_name.as_str(),
                MIN_IDLE_TIME.as_millis() as u64,
                "0-0",
                StreamAutoClaimOptions::default().count(max_jobs),
            )
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

        let mut pipeline = redis::pipe();
        pipeline
            .atomic()
            .ignore()
            .xack(
                self.queue_name.as_str(),
                self.consumer_group.as_str(),
                &[stream_id],
            )
            .xdel(self.queue_name.as_str(), &[stream_id]);

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
                    let enqueue_cmd = self.enqueue_redis_cmd(&job_data, Some(backoff_timestamp))?;

                    pipeline.add_command(enqueue_cmd);

                    false // Do not delete the context if we were able to retry the job one more time
                } else {
                    true // We hit the maximum amount of retries, we won't re-enqueue the job, so we can just remove the context
                }
            }
            Outcome::Success => true, // Execution succeeded, we don't need the context anymore
        };

        {
            let mut conn = self.redis_pool.get();
            pipeline.query_async::<_, ()>(&mut conn).await?;
        }

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

        let mut conn = self.redis_pool.get();
        conn.xclaim(
            self.queue_name.as_str(),
            self.consumer_group.as_str(),
            self.consumer_name.as_str(),
            0,
            &[stream_id],
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
