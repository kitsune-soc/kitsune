#[macro_use]
extern crate tracing;

use self::job::Job;
use async_trait::async_trait;
use athena::JobContextRepository;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures_util::{stream::BoxStream, StreamExt, TryStreamExt};
use kitsune_db::{
    json::Json,
    model::job_context::{JobContext, NewJobContext},
    schema::job_context,
    PgPool,
};
use speedy_uuid::Uuid;

mod job;

pub struct KitsuneContextRepo {
    db_pool: PgPool,
}

impl KitsuneContextRepo {
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
