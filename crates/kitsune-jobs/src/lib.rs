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
use athena::{JobContextRepository, Runnable};
use derive_more::From;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures_util::{StreamExt, TryStreamExt, stream::BoxStream};
use kitsune_core::traits::Deliverer;
use kitsune_db::{
    PgPool, insert::NewJobContext, json::Json, model::JobContext, schema::job_context,
    with_connection,
};
use kitsune_email::MailingService;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

pub mod deliver;
pub mod mailing;

pub struct Service {
    pub mailing: MailingService,
}

pub struct JobRunnerContext {
    pub deliverer: Box<dyn Deliverer>,
    pub db_pool: PgPool,
    pub service: Service,
}

#[derive(Debug, Deserialize, From, Serialize)]
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

impl Runnable for Job {
    type Context = JobRunnerContext;
    type Error = kitsune_error::Error;

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

    pub async fn remove_context_ops<C>(
        conn: &mut C,
        job_id: Uuid,
    ) -> Result<(), diesel::result::Error>
    where
        C: diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
    {
        diesel::delete(job_context::table.find(job_id))
            .execute(conn)
            .await?;

        Ok(())
    }
}

impl JobContextRepository for KitsuneContextRepo {
    type JobContext = Job;
    type Error = kitsune_error::Error;
    type Stream = BoxStream<'static, Result<(Uuid, Self::JobContext), Self::Error>>;

    async fn fetch_context<I>(&self, job_ids: I) -> Result<Self::Stream, Self::Error>
    where
        I: Iterator<Item = Uuid> + Send + 'static,
    {
        let stream = with_connection!(self.db_pool, |conn| {
            job_context::table
                .filter(job_context::id.eq_any(job_ids))
                .load_stream::<JobContext<Job>>(conn)
                .await
        })?;

        Ok(stream
            .map_ok(|ctx| (ctx.id, ctx.context.0))
            .map_err(kitsune_error::Error::from)
            .boxed())
    }

    async fn remove_context(&self, job_id: Uuid) -> Result<(), Self::Error> {
        with_connection!(self.db_pool, |conn| {
            Self::remove_context_ops(conn, job_id).await
        })?;

        Ok(())
    }

    async fn store_context(
        &self,
        job_id: Uuid,
        context: Self::JobContext,
    ) -> Result<(), Self::Error> {
        with_connection!(self.db_pool, |conn| {
            diesel::insert_into(job_context::table)
                .values(NewJobContext {
                    id: job_id,
                    context: Json(context),
                })
                .execute(conn)
                .await
        })?;

        Ok(())
    }
}
