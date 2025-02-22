use bytes::Bytes;
use diesel_async::{AsyncConnection, AsyncPgConnection, SimpleAsyncConnection};
use fred::{
    clients::Pool as RedisPool, interfaces::ClientLike, types::config::Config as RedisConfig,
};
use http::header::CONTENT_TYPE;
use http_body_util::Full;
use isolang::Language;
use kitsune_config::{
    database::Configuration as DatabaseConfig,
    language_detection::{self, DetectionBackend},
};
use kitsune_db::PgPool;
use resource::provide_resource;
use std::env;
use triomphe::Arc;
use url::Url;
use uuid::Uuid;

mod catch_panic;
mod macros;
mod redis;
mod resource;

pub fn build_ap_response<B>(body: B) -> http::Response<Full<Bytes>>
where
    Bytes: From<B>,
{
    http::Response::builder()
        .header(CONTENT_TYPE, "application/activity+json")
        .body(Full::new(body.into()))
        .unwrap()
}

pub async fn database_test<F, O>(func: F) -> O
where
    F: AsyncFnOnce(PgPool) -> O,
{
    let db_url = env::var("DATABASE_URL").unwrap();
    let mut url = Url::parse(&db_url).unwrap();

    // Create a new separate database for this test
    let id = Uuid::new_v4().as_simple().to_string();
    let db_name = format!("kitsune_test_{id}");

    let mut admin_conn = AsyncPgConnection::establish(url.as_str()).await.unwrap();

    admin_conn
        .batch_execute(&format!(
            "CREATE DATABASE {db_name} ENCODING 'UTF8' TEMPLATE template0"
        ))
        .await
        .unwrap();

    url.set_path(&db_name);

    let pool = kitsune_db::connect(&DatabaseConfig {
        url: url.as_str().into(),
        max_connections: 10,
        use_tls: false,
    })
    .await
    .expect("Failed to connect to database");

    provide_resource(pool, func, async |_pool| {
        // Drop the newly created database. We don't need it anymore.
        if let Err(error) = admin_conn
            .batch_execute(&format!("DROP DATABASE {db_name}"))
            .await
        {
            eprintln!("failed to drop database. sorry. error: {error:?}");
        }
    })
    .await
}

#[must_use]
pub fn language_detection_config() -> language_detection::Configuration {
    language_detection::Configuration {
        backend: DetectionBackend::Whichlang,
        default_language: Language::Eng,
    }
}

pub async fn minio_test<F, O>(func: F) -> O
where
    F: AsyncFnOnce(Arc<kitsune_s3::Client>) -> O,
{
    let endpoint = env::var("MINIO_URL").unwrap();
    let endpoint = endpoint.parse().unwrap();

    // Create a new bucket with a random ID
    let bucket_id = Uuid::new_v4().as_simple().to_string();
    let bucket = rusty_s3::Bucket::new(
        endpoint,
        rusty_s3::UrlStyle::Path,
        format!("test-bucket-{bucket_id}"),
        "us-east-1",
    )
    .unwrap();
    let credentials = rusty_s3::Credentials::new("minioadmin", "minioadmin");
    let client = kitsune_s3::Client::builder()
        .bucket(bucket)
        .credentials(credentials)
        .build();
    let client = Arc::new(client);

    client.create_bucket().await.unwrap();

    provide_resource(client, func, async |client| {
        client.delete_bucket().await.unwrap();
    })
    .await
}

pub async fn redis_test<F, O>(func: F) -> O
where
    F: AsyncFnOnce(RedisPool) -> O,
{
    let redis_url = env::var("REDIS_URL").unwrap();
    let mut config = RedisConfig::from_url(&redis_url).unwrap();

    // Connect to a random Redis database
    let db_id = self::redis::find_unused_database(&config).await;
    config.database = Some(db_id);
    let pool = RedisPool::new(config, None, None, None, 5).unwrap();
    pool.init().await.unwrap();

    provide_resource(pool, func, async |pool| {
        pool.custom::<(), ()>(fred::cmd!("FLUSHDB"), vec![])
            .await
            .unwrap();
    })
    .await
}
