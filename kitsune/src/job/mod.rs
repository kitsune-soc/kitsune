use self::deliver::{
    accept::DeliverAccept, create::DeliverCreate, delete::DeliverDelete,
    favourite::DeliverFavourite, follow::DeliverFollow, unfavourite::DeliverUnfavourite,
    unfollow::DeliverUnfollow, update::DeliverUpdate,
};
use crate::{activitypub::Deliverer, error::Result, state::Zustand};
use async_trait::async_trait;
use athena::Runnable;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::task::JoinSet;

mod catch_panic;
pub mod deliver;

const PAUSE_BETWEEN_QUERIES: Duration = Duration::from_secs(5);
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

pub struct JobContext {
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
    type Context = JobContext;
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

#[instrument(skip(state))]
pub async fn run_dispatcher(state: Zustand, num_job_workers: usize) {
    let deliverer = Deliverer::builder()
        .federation_filter(state.service.federation_filter.clone())
        .build();

    let mut executor = JoinSet::new();
    let mut pause_between_queries = tokio::time::interval(PAUSE_BETWEEN_QUERIES);
    let mut do_pause = false;

    loop {
        if do_pause {
            pause_between_queries.tick().await;
            do_pause = false;
        }

        let num_jobs = num_job_workers - executor.len();
        let jobs = match get_jobs(&state.db_conn, num_jobs).await {
            Ok(jobs) => jobs,
            Err(err) => {
                error!(error = %err, "Failed to get jobs from database");
                continue;
            }
        };

        if jobs.is_empty() && executor.is_empty() {
            do_pause = true;
            continue;
        }

        for job in jobs {
            executor.spawn(execute_one(job, state.clone(), deliverer.clone()));
        }

        if tokio::time::timeout(EXECUTION_TIMEOUT_DURATION, executor.join_next())
            .await
            .is_err()
        {
            debug!("Reached timeout. Waiting for job");
        }
    }
}
