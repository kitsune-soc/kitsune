use self::deliver::{
    accept::DeliverAccept, create::DeliverCreate, delete::DeliverDelete,
    favourite::DeliverFavourite, follow::DeliverFollow, unfavourite::DeliverUnfavourite,
    unfollow::DeliverUnfollow, update::DeliverUpdate,
};
use crate::{activitypub::Deliverer, error::Result, state::Zustand};
use async_trait::async_trait;
use athena::{JobContextRepository, JobQueue, Runnable};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures_util::{stream::BoxStream, StreamExt, TryStreamExt};
use kitsune_db::{
    json::Json,
    model::job_context::{JobContext, NewJobContext},
    schema::job_context,
    PgPool,
};
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;
use std::{sync::Arc, time::Duration};
use tokio::task::JoinSet;
use typed_builder::TypedBuilder;

mod catch_panic;
pub mod deliver;

const EXECUTION_TIMEOUT_DURATION: Duration = Duration::from_secs(30);
const MAX_CONCURRENT_REQUESTS: usize = 10;

macro_rules! impl_from {
    (
        $(#[$top_annotation:meta])*
        $vb:vis enum $name:ident {
        $(
            $(#[$branch_annotation:meta])*
            $branch_name:ident ($from_type:ty)
        ),+
        $(,)*
    }) => {
        $(#[$top_annotation])*
        $vb enum $name {
            $(
                $(#[$branch_annotation])*
                $branch_name($from_type),
            )*
        }

        $(
            impl From<$from_type> for $name {
                fn from(val: $from_type) -> Self {
                    Self::$branch_name(val)
                }
            }
        )*
    };
}

pub struct JobRunnerContext {
    deliverer: Deliverer,
    state: Zustand,
}

impl_from! {
    #[derive(Debug, Deserialize, Serialize)]
    pub enum Job {
        DeliverAccept(DeliverAccept),
        DeliverCreate(DeliverCreate),
        DeliverDelete(DeliverDelete),
        DeliverFavourite(DeliverFavourite),
        DeliverFollow(DeliverFollow),
        DeliverUnfavourite(DeliverUnfavourite),
        DeliverUnfollow(DeliverUnfollow),
        DeliverUpdate(DeliverUpdate),
    }
}

#[async_trait]
impl Runnable for Job {
    type Context = JobRunnerContext;
    type Error = anyhow::Error;

    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        match self {
            Self::DeliverAccept(job) => job.run(ctx).await,
            Self::DeliverCreate(job) => job.run(ctx).await,
            Self::DeliverDelete(job) => job.run(ctx).await,
            Self::DeliverFavourite(job) => job.run(ctx).await,
            Self::DeliverFollow(job) => job.run(ctx).await,
            Self::DeliverUnfavourite(job) => job.run(ctx).await,
            Self::DeliverUnfollow(job) => job.run(ctx).await,
            Self::DeliverUpdate(job) => job.run(ctx).await,
        }
    }
}

#[derive(TypedBuilder)]
pub struct KitsuneContextRepo {
    db_pool: PgPool,
}

impl KitsuneContextRepo {
    #[must_use]
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl JobContextRepository for KitsuneContextRepo {
    type JobContext = Job;
    type Error = anyhow::Error;
    type Stream = BoxStream<'static, Result<(Uuid, Self::JobContext), Self::Error>>;

    async fn fetch_context<I>(&self, job_ids: I) -> Result<Self::Stream, Self::Error>
    where
        I: Iterator<Item = Uuid> + Send + 'static,
    {
        let mut conn = self.db_pool.get().await?;
        let stream = job_context::table
            .filter(job_context::id.eq_any(job_ids))
            .load_stream::<JobContext<Job>>(&mut conn)
            .await?;

        Ok(stream
            .map_ok(|ctx| (ctx.id, ctx.context.0))
            .map_err(anyhow::Error::from)
            .boxed())
    }

    async fn remove_context(&self, job_id: Uuid) -> Result<(), Self::Error> {
        let mut conn = self.db_pool.get().await?;
        diesel::delete(job_context::table.find(job_id))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    async fn store_context(
        &self,
        job_id: Uuid,
        context: Self::JobContext,
    ) -> Result<(), Self::Error> {
        let mut conn = self.db_pool.get().await?;
        diesel::insert_into(job_context::table)
            .values(NewJobContext {
                id: job_id,
                context: Json(context),
            })
            .execute(&mut conn)
            .await?;

        Ok(())
    }
}

#[instrument(skip(job_queue, state))]
pub async fn run_dispatcher(
    job_queue: JobQueue<KitsuneContextRepo>,
    state: Zustand,
    num_job_workers: usize,
) {
    let deliverer = Deliverer::builder()
        .federation_filter(state.service.federation_filter.clone())
        .build();
    let ctx = Arc::new(JobRunnerContext { deliverer, state });

    let mut job_joinset = JoinSet::new();
    loop {
        while let Err(error) = job_queue
            .spawn_jobs(
                num_job_workers - job_joinset.len(),
                Arc::clone(&ctx),
                &mut job_joinset,
            )
            .await
        {
            error!(?error, "failed to spawn more jobs");
            just_retry::sleep_a_bit().await;
        }

        let join_all = async { while job_joinset.join_next().await.is_some() {} };
        let _ = tokio::time::timeout(EXECUTION_TIMEOUT_DURATION, join_all).await;
    }
}
