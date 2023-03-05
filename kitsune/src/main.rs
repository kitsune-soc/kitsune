#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

use axum_prometheus::{AXUM_HTTP_REQUESTS_DURATION_SECONDS, SECONDS_DURATION_BUCKETS};
use kitsune::{
    activitypub::Fetcher,
    config::Configuration,
    http, job,
    resolve::PostResolver,
    service::{
        account::AccountService, attachment::AttachmentService, oauth2::Oauth2Service,
        post::PostService, search::GrpcSearchService, timeline::TimelineService, user::UserService,
    },
    state::{EventEmitter, Service, Zustand},
    webfinger::Webfinger,
};
use kitsune_http_client::Client;
use kitsune_messaging::{redis::RedisMessagingBackend, MessagingHub};
use kitsune_storage::fs::Storage as FsStorage;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Layer as _;
use sea_orm::DatabaseConnection;
use std::{env, future, sync::Arc};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, Layer as _, Registry};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const STARTUP_FIGLET: &str = r#"
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃                                                           ┃
┃  ██╗  ██╗██╗████████╗███████╗██╗   ██╗███╗   ██╗███████╗  ┃
┃  ██║ ██╔╝██║╚══██╔══╝██╔════╝██║   ██║████╗  ██║██╔════╝  ┃
┃  █████╔╝ ██║   ██║   ███████╗██║   ██║██╔██╗ ██║█████╗    ┃
┃  ██╔═██╗ ██║   ██║   ╚════██║██║   ██║██║╚██╗██║██╔══╝    ┃
┃  ██║  ██╗██║   ██║   ███████║╚██████╔╝██║ ╚████║███████╗  ┃
┃  ╚═╝  ╚═╝╚═╝   ╚═╝   ╚══════╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝  ┃
┃                                                           ┃
┃            ActivityPub-federated microblogging            ┃
┃                                                           ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
"#;

fn initialise_logging(config: &Configuration) {
    let (prometheus_recorder, server_future) = PrometheusBuilder::new()
        // Some defaults that would have been set by the `axum-prometheus` crate
        .set_buckets_for_metric(
            Matcher::Full(AXUM_HTTP_REQUESTS_DURATION_SECONDS.to_string()),
            SECONDS_DURATION_BUCKETS,
        )
        .unwrap()
        .with_http_listener(([0, 0, 0, 0], config.prometheus_port))
        .build()
        .unwrap();
    tokio::spawn(server_future);

    let env_filter = env::var("RUST_LOG").map_or_else(
        |_| Targets::default().with_default(LevelFilter::INFO),
        |targets| targets.parse().expect("Failed to parse RUST_LOG value"),
    );
    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
        .with(MetricsLayer::new());
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let recorder = TracingContextLayer::all().layer(prometheus_recorder);
    metrics::set_boxed_recorder(Box::new(recorder)).unwrap();
}

async fn initialise_state(
    config: &Configuration,
    conn: DatabaseConnection,
    redis_conn: deadpool_redis::Pool,
) -> Zustand {
    let redis_messaging_backend = RedisMessagingBackend::new(&config.redis_url)
        .await
        .expect("Failed to construct messaging backend");
    let messaging_hub = MessagingHub::new(redis_messaging_backend);
    let status_event_emitter = messaging_hub.emitter("event.status".into());

    let search_service =
        GrpcSearchService::new(&config.search_index_server, &config.search_servers)
            .await
            .expect("Failed to connect to the search servers");

    let fetcher = Fetcher::with_defaults(conn.clone(), search_service.clone(), redis_conn.clone());
    let webfinger = Webfinger::with_defaults(redis_conn.clone());

    let account_service = AccountService::builder()
        .db_conn(conn.clone())
        .build()
        .unwrap();

    let attachment_service = AttachmentService::builder()
        .client(
            Client::builder()
                .content_length_limit(None)
                .user_agent(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION")
                ))
                .unwrap()
                .build(),
        )
        .domain(config.domain.clone())
        .db_conn(conn.clone())
        .media_proxy_enabled(config.media_proxy_enabled)
        .storage_backend(Arc::new(FsStorage::new(config.upload_dir.clone())))
        .build()
        .unwrap();

    let oauth2_service = Oauth2Service::builder()
        .db_conn(conn.clone())
        .build()
        .unwrap();

    let post_resolver = PostResolver::new(conn.clone(), fetcher.clone(), webfinger.clone());
    let post_service = PostService::builder()
        .db_conn(conn.clone())
        .domain(config.domain.clone())
        .post_resolver(post_resolver)
        .search_service(Arc::new(search_service.clone()))
        .status_event_emitter(status_event_emitter.clone())
        .build()
        .unwrap();

    let timeline_service = TimelineService::builder()
        .db_conn(conn.clone())
        .build()
        .unwrap();

    let user_service = UserService::builder()
        .domain(config.domain.clone())
        .db_conn(conn.clone())
        .build()
        .unwrap();

    #[cfg(feature = "mastodon-api")]
    let mastodon_mapper = kitsune::mapping::MastodonMapper::with_defaults(
        attachment_service.clone(),
        conn.clone(),
        config.default_avatar_url(),
        redis_conn,
        status_event_emitter
            .consumer()
            .await
            .expect("Failed to register status event consumer"),
    );

    Zustand {
        config: config.clone(),
        db_conn: conn,
        event_emitter: EventEmitter {
            post: status_event_emitter.clone(),
        },
        fetcher,
        #[cfg(feature = "mastodon-api")]
        mastodon_mapper,
        service: Service {
            account: account_service,
            oauth2: oauth2_service,
            search: Arc::new(search_service),
            post: post_service,
            timeline: timeline_service,
            attachment: attachment_service,
            user: user_service,
        },
        webfinger,
    }
}

#[tokio::main]
async fn main() {
    println!("{STARTUP_FIGLET}");

    dotenvy::dotenv().ok();
    let config: Configuration = envy::from_env().expect("Failed to parse configuration");
    initialise_logging(&config);

    let conn = kitsune_db::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let redis_manager = deadpool_redis::Manager::new(config.redis_url.clone())
        .expect("Failed to build Redis pool manager");
    let redis_conn = deadpool_redis::Pool::builder(redis_manager)
        .build()
        .expect("Failed to build Redis pool");

    let state = initialise_state(&config, conn, redis_conn).await;

    tokio::spawn(self::http::run(state.clone(), config.port));

    for _ in 0..config.job_workers.get() {
        tokio::spawn(self::job::run(state.clone()));
    }

    future::pending::<()>().await;
}
