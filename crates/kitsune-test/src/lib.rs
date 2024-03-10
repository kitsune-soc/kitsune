use self::catch_panic::CatchPanic;
use bytes::Bytes;
use diesel_async::RunQueryDsl;
use futures_util::Future;
use http::header::CONTENT_TYPE;
use http_body_util::Full;
use isolang::Language;
use kitsune_config::{
    database::Configuration as DatabaseConfig,
    language_detection::{self, DetectionBackend},
};
use kitsune_db::PgPool;
use multiplex_pool::RoundRobinStrategy;
use redis::aio::ConnectionManager;
use scoped_futures::ScopedFutureExt;
use std::{error::Error, panic};

mod catch_panic;
mod container;
mod resource;

type BoxError = Box<dyn Error + Send + Sync>;

pub fn build_ap_response<B>(body: B) -> http::Response<Full<Bytes>>
where
    Bytes: From<B>,
{
    http::Response::builder()
        .header(CONTENT_TYPE, "application/activity+json")
        .body(Full::new(body.into()))
        .unwrap()
}

pub async fn database_test<F, Fut>(func: F) -> Fut::Output
where
    F: FnOnce(PgPool) -> Fut,
    Fut: Future,
{
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let resource_handle = get_resource!("DATABASE_URL", self::container::postgres);
    let pool = kitsune_db::connect(&DatabaseConfig {
        url: resource_handle.url().into(),
        max_connections: 10,
        use_tls: false,
    })
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

#[must_use]
pub fn language_detection_config() -> language_detection::Configuration {
    language_detection::Configuration {
        backend: DetectionBackend::Whichlang,
        default_language: Language::Eng,
    }
}

pub async fn redis_test<F, Fut>(func: F) -> Fut::Output
where
    F: FnOnce(multiplex_pool::Pool<ConnectionManager>) -> Fut,
    Fut: Future,
{
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let resource_handle = get_resource!("REDIS_URL", self::container::redis);
    let client = redis::Client::open(resource_handle.url().as_ref()).unwrap();
    let pool = multiplex_pool::Pool::from_producer(
        || client.get_connection_manager(),
        5,
        RoundRobinStrategy::default(),
    )
    .await
    .unwrap();

    let out = CatchPanic::new(func(pool.clone())).await;

    let mut conn = pool.get();
    let (): () = redis::cmd("FLUSHALL").query_async(&mut conn).await.unwrap();

    match out {
        Ok(out) => out,
        Err(err) => panic::resume_unwind(err),
    }
}
