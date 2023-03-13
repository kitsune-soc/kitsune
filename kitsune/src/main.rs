#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

use axum_prometheus::{AXUM_HTTP_REQUESTS_DURATION_SECONDS, SECONDS_DURATION_BUCKETS};
use kitsune::{
    activitypub::Fetcher,
    cache::{ArcCache, NoopCache},
    config::{CacheConfiguration, Configuration, MessagingConfiguration, StorageConfiguration},
    http, job,
    resolve::PostResolver,
    service::{
        account::AccountService, attachment::AttachmentService, oauth2::Oauth2Service,
        post::PostService, search::GrpcSearchService, timeline::TimelineService, url::UrlService,
        user::UserService,
    },
    state::{EventEmitter, Service, Zustand},
    webfinger::Webfinger,
};
use kitsune_messaging::{
    redis::RedisMessagingBackend, tokio_broadcast::TokioBroadcastMessagingBackend, MessagingHub,
};
use kitsune_storage::{fs::Storage as FsStorage, StorageBackend};
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
        .with_http_listener(([0, 0, 0, 0], config.server.prometheus_port))
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

async fn prepare_cache<K, V>(config: &Configuration, cache_name: &str) -> ArcCache<K, V>
where
    K: Send + Sync + ?Sized,
    V: Send + Sync,
{
    match config.cache {
        CacheConfiguration::InMemory => {
            // TODO: Implement in-memory cache
            todo!();
        }
        CacheConfiguration::None => Arc::new(NoopCache),
        CacheConfiguration::Redis(ref redis_config) => {
            // TODO: Create new cache with redis connection pool reuse
            todo!();
        }
    }
}

async fn prepare_storage(config: &Configuration) -> Arc<dyn StorageBackend> {
    match config.storage {
        StorageConfiguration::Fs(ref fs_config) => {
            Arc::new(FsStorage::new(fs_config.upload_dir.as_str().into()))
        }
        StorageConfiguration::S3(ref s3_config) => {
            // TODO: Connect to S3
            todo!();
        }
    }
}

async fn prepare_messaging(config: &Configuration) -> MessagingHub {
    match config.messaging {
        MessagingConfiguration::InProcess => {
            MessagingHub::new(TokioBroadcastMessagingBackend::default())
        }
        MessagingConfiguration::Redis(ref redis_config) => {
            let redis_messaging_backend = RedisMessagingBackend::new(&redis_config.redis_url)
                .await
                .expect("Failed to construct messaging backend");

            MessagingHub::new(redis_messaging_backend)
        }
    }
}

async fn initialise_state(config: &Configuration, conn: DatabaseConnection) -> Zustand {
    let messaging_hub = prepare_messaging(config).await;
    let status_event_emitter = messaging_hub.emitter("event.status".into());

    let search_service =
        GrpcSearchService::new(&config.search.index_server, &config.search.search_servers)
            .await
            .expect("Failed to connect to the search servers");

    let fetcher = Fetcher::with_defaults(conn.clone(), search_service.clone(), todo!());
    let webfinger = Webfinger::with_defaults(todo!());

    let url_service = UrlService::builder()
        .schema(config.url.schema.as_str())
        .domain(config.url.domain.as_str())
        .build()
        .unwrap();

    let account_service = AccountService::builder()
        .db_conn(conn.clone())
        .build()
        .unwrap();

    let storage_backend = prepare_storage(config).await;
    let attachment_service = AttachmentService::builder()
        .db_conn(conn.clone())
        .media_proxy_enabled(config.server.media_proxy_enabled)
        .storage_backend(storage_backend)
        .url_service(url_service.clone())
        .build()
        .unwrap();

    let oauth2_service = Oauth2Service::builder()
        .db_conn(conn.clone())
        .build()
        .unwrap();

    let post_resolver = PostResolver::new(conn.clone(), fetcher.clone(), webfinger.clone());
    let post_service = PostService::builder()
        .db_conn(conn.clone())
        .post_resolver(post_resolver)
        .search_service(Arc::new(search_service.clone()))
        .status_event_emitter(status_event_emitter.clone())
        .url_service(url_service.clone())
        .build()
        .unwrap();

    let timeline_service = TimelineService::builder()
        .db_conn(conn.clone())
        .build()
        .unwrap();

    let user_service = UserService::builder()
        .db_conn(conn.clone())
        .url_service(url_service.clone())
        .build()
        .unwrap();

    #[cfg(feature = "mastodon-api")]
    let mastodon_mapper = kitsune::mapping::MastodonMapper::with_defaults(
        attachment_service.clone(),
        conn.clone(),
        status_event_emitter
            .consumer()
            .await
            .expect("Failed to register status event consumer"),
        todo!(),
        url_service.clone(),
    );

    Zustand {
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
            url: url_service,
            user: user_service,
        },
        webfinger,
    }
}

#[tokio::main]
async fn main() {
    println!("{STARTUP_FIGLET}");

    let config = match Configuration::load("config.dhall") {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };
    initialise_logging(&config);

    let conn = kitsune_db::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");
    let state = initialise_state(&config, conn).await;

    tokio::spawn(self::http::run(state.clone(), config.server.clone()));

    for _ in 0..config.server.job_workers {
        tokio::spawn(self::job::run(state.clone()));
    }

    future::pending::<()>().await;
}
