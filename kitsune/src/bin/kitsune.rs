#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

use axum_prometheus::{AXUM_HTTP_REQUESTS_DURATION_SECONDS, SECONDS_DURATION_BUCKETS};
use kitsune::{
    activitypub::Fetcher, config::Configuration, db, http, job, search::GrpcSearchService,
    state::Zustand, webfinger::Webfinger,
};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use std::future;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config: Configuration = envy::from_env().expect("Failed to parse configuration");
    let conn = self::db::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    PrometheusBuilder::new()
        // Some defaults that would have been set by the `axum-prometheus` crate
        .set_buckets_for_metric(
            Matcher::Full(AXUM_HTTP_REQUESTS_DURATION_SECONDS.to_string()),
            SECONDS_DURATION_BUCKETS,
        )
        .unwrap()
        .with_http_listener(([0, 0, 0, 0], config.prometheus_port))
        .install()
        .expect("Failed to install Prometheus metrics recorder");

    let redis_manager = deadpool_redis::Manager::new(config.redis_url.clone())
        .expect("Failed to build Redis pool manager");
    let redis_conn = deadpool_redis::Pool::builder(redis_manager)
        .build()
        .expect("Failed to build Redis pool");

    let search_service =
        GrpcSearchService::new(&config.search_index_server, &config.search_servers)
            .await
            .expect("Failed to connect to the search servers");

    let state = Zustand {
        config: config.clone(),
        db_conn: conn.clone(),
        fetcher: Fetcher::with_defaults(conn, search_service.clone(), redis_conn.clone()),
        search_service,
        webfinger: Webfinger::with_redis_cache(redis_conn),
    };

    tokio::spawn(self::http::run(state.clone(), config.port));

    for _ in 0..config.job_workers.get() {
        tokio::spawn(self::job::run(state.clone()));
    }

    future::pending::<()>().await;
}
