use self::catch_panic::CatchPanic;
use diesel_async::RunQueryDsl;
use futures_util::Future;
use kitsune_db::PgPool;
use scoped_futures::ScopedFutureExt;
use std::{env, error::Error, panic};

mod catch_panic;

type BoxError = Box<dyn Error + Send + Sync>;

pub async fn database_test<F, Fut>(func: F) -> Fut::Output
where
    F: FnOnce(PgPool) -> Fut,
    Fut: Future,
{
    let db_url = env::var("DATABASE_URL").expect("Missing database URL");
    let pool = kitsune_db::connect(&db_url, 10)
        .await
        .expect("Failed to connect to database");

    let out = CatchPanic::new(func(pool.clone())).await;

    pool.with_connection(|db_conn| {
        async move {
            diesel::sql_query("DROP SCHEMA public CASCADE")
                .execute(db_conn)
                .await
                .expect("Failed to delete schema");

            diesel::sql_query("CREATE SCHEMA public")
                .execute(db_conn)
                .await
                .expect("Failed to create schema");

            Ok::<_, BoxError>(())
        }
        .scoped()
    })
    .await
    .expect("Failed to get connection");

    match out {
        Ok(out) => out,
        Err(err) => panic::resume_unwind(err),
    }
}

pub async fn redis_test<F, Fut>(func: F) -> Fut::Output
where
    F: FnOnce(deadpool_redis::Pool) -> Fut,
    Fut: Future,
{
    let redis_url = env::var("REDIS_URL").expect("Missing redis URL");
    let config = deadpool_redis::Config::from_url(redis_url);
    let pool = config
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .unwrap();

    let out = CatchPanic::new(func(pool.clone())).await;

    let mut conn = pool.get().await.unwrap();
    let (): () = redis::cmd("FLUSHALL").query_async(&mut conn).await.unwrap();

    match out {
        Ok(out) => out,
        Err(err) => panic::resume_unwind(err),
    }
}
