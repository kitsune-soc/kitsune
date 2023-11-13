#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(forbidden_lint_groups)]

#[macro_use]
extern crate tracing;

use self::{
    deliver::{
        accept::DeliverAccept, create::DeliverCreate, delete::DeliverDelete,
        favourite::DeliverFavourite, follow::DeliverFollow, reject::DeliverReject,
        unfavourite::DeliverUnfavourite, unfollow::DeliverUnfollow, update::DeliverUpdate,
    },
    mailing::confirmation::SendConfirmationMail,
};
use crate::error::Result;
use athena::{JobContextRepository, Runnable};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures_util::{stream::BoxStream, StreamExt, TryStreamExt};
use kitsune_db::{
    json::Json,
    model::job_context::{JobContext, NewJobContext},
    schema::job_context,
    PgPool,
};
use kitsune_util::impl_from;
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

pub mod deliver;
pub mod error;
pub mod mailing;

const MAX_CONCURRENT_REQUESTS: usize = 10;

pub struct JobRunnerContext {
    //pub deliverer: Deliverer,
    pub db_pool: PgPool,
}

impl_from! {
    #[derive(Debug, Deserialize, Serialize)]
    pub enum Job {
        DeliverAccept(DeliverAccept),
        DeliverCreate(DeliverCreate),
        DeliverDelete(DeliverDelete),
        DeliverFavourite(DeliverFavourite),
        DeliverFollow(DeliverFollow),
        DeliverReject(DeliverReject),
        DeliverUnfavourite(DeliverUnfavourite),
        DeliverUnfollow(DeliverUnfollow),
        DeliverUpdate(DeliverUpdate),
        SendConfirmationMail(SendConfirmationMail),
    }
}

impl Runnable for Job {
    type Context = JobRunnerContext;
    type Error = eyre::Report;

    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        match self {
            Self::DeliverAccept(job) => job.run(ctx).await,
            Self::DeliverCreate(job) => job.run(ctx).await,
            Self::DeliverDelete(job) => job.run(ctx).await,
            Self::DeliverFavourite(job) => job.run(ctx).await,
            Self::DeliverFollow(job) => job.run(ctx).await,
            Self::DeliverReject(job) => job.run(ctx).await,
            Self::DeliverUnfavourite(job) => job.run(ctx).await,
            Self::DeliverUnfollow(job) => job.run(ctx).await,
            Self::DeliverUpdate(job) => job.run(ctx).await,
            Self::SendConfirmationMail(job) => job.run(ctx).await,
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

impl JobContextRepository for KitsuneContextRepo {
    type JobContext = Job;
    type Error = eyre::Report;
    type Stream = BoxStream<'static, Result<(Uuid, Self::JobContext), Self::Error>>;

    async fn fetch_context<I>(&self, job_ids: I) -> Result<Self::Stream, Self::Error>
    where
        I: Iterator<Item = Uuid> + Send + 'static,
    {
        let stream = self
            .db_pool
            .with_connection(|conn| {
                job_context::table
                    .filter(job_context::id.eq_any(job_ids))
                    .load_stream::<JobContext<Job>>(conn)
                    .scoped()
            })
            .await?;

        Ok(stream
            .map_ok(|ctx| (ctx.id, ctx.context.0))
            .map_err(eyre::Report::from)
            .boxed())
    }

    async fn remove_context(&self, job_id: Uuid) -> Result<(), Self::Error> {
        self.db_pool
            .with_connection(|conn| {
                diesel::delete(job_context::table.find(job_id))
                    .execute(conn)
                    .scoped()
            })
            .await?;

        Ok(())
    }

    async fn store_context(
        &self,
        job_id: Uuid,
        context: Self::JobContext,
    ) -> Result<(), Self::Error> {
        self.db_pool
            .with_connection(|conn| {
                diesel::insert_into(job_context::table)
                    .values(NewJobContext {
                        id: job_id,
                        context: Json(context),
                    })
                    .execute(conn)
                    .scoped()
            })
            .await?;

        Ok(())
    }
}
