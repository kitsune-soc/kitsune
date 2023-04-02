#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use axum_prometheus::{AXUM_HTTP_REQUESTS_DURATION_SECONDS, SECONDS_DURATION_BUCKETS};
use futures_util::future::OptionFuture;
use kitsune::{
    activitypub::Fetcher,
    cache::{ArcCache, InMemoryCache, NoopCache, RedisCache},
    config::{
        CacheConfiguration, Configuration, MessagingConfiguration, OidcConfiguration,
        SearchConfiguration, StorageConfiguration,
    },
    http, job,
    resolve::PostResolver,
    service::{
        account::AccountService,
        attachment::AttachmentService,
        instance::InstanceService,
        job::JobService,
        oauth2::Oauth2Service,
        oidc::{async_client, OidcService},
        post::PostService,
        search::{GrpcSearchService, NoopSearchService, SearchService, SqlSearchService},
        timeline::TimelineService,
        url::UrlService,
        user::UserService,
    },
    state::{EventEmitter, Service, Zustand},
    webfinger::Webfinger,
};
use kitsune_messaging::{
    redis::RedisMessagingBackend, tokio_broadcast::TokioBroadcastMessagingBackend, MessagingHub,
};
use kitsune_storage::{fs::Storage as FsStorage, s3::Storage as S3Storage, Storage};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Layer as _;
use once_cell::sync::OnceCell;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    ClientId, ClientSecret, IssuerUrl, RedirectUrl,
};
use sea_orm::{ConnectOptions, DatabaseConnection};
use serde::{de::DeserializeOwned, Serialize};
use std::{env, fmt::Display, future, process, sync::Arc, time::Duration};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, Layer as _, Registry};

#[cfg(feature = "meilisearch")]
use kitsune::service::search::MeiliSearchService;

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

fn prepare_cache<K, V>(config: &Configuration, cache_name: &str) -> ArcCache<K, V>
where
    K: Display + Send + Sync + ?Sized + 'static,
    V: Clone + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    let cache = match config.cache {
        CacheConfiguration::InMemory => InMemoryCache::new(100, Duration::from_secs(60)).into(), // TODO: Parameterise this
        CacheConfiguration::None => NoopCache.into(),
        CacheConfiguration::Redis(ref redis_config) => {
            static REDIS_POOL: OnceCell<deadpool_redis::Pool> = OnceCell::new();

            let pool = REDIS_POOL.get_or_init(|| {
                let config = deadpool_redis::Config::from_url(&redis_config.redis_url);
                config
                    .create_pool(Some(deadpool_redis::Runtime::Tokio1))
                    .unwrap()
            });

            RedisCache::builder()
                .prefix(cache_name)
                .redis_conn(pool.clone())
                .ttl(Duration::from_secs(60)) // TODO: Parameterise this
                .build()
                .unwrap()
                .into()
        }
    };

    Arc::new(cache)
}

fn prepare_storage(config: &Configuration) -> Storage {
    match config.storage {
        StorageConfiguration::Fs(ref fs_config) => {
            FsStorage::new(fs_config.upload_dir.as_str().into()).into()
        }
        StorageConfiguration::S3(ref s3_config) => {
            let s3_client_config = aws_sdk_s3::Config::builder()
                .region(Region::new(s3_config.region.clone()))
                .endpoint_url(s3_config.endpoint_url.as_str())
                .force_path_style(s3_config.force_path_style)
                .credentials_provider(Credentials::from_keys(
                    s3_config.access_key.as_str(),
                    s3_config.secret_access_key.as_str(),
                    None,
                ))
                .build();

            S3Storage::new(s3_config.bucket_name.clone(), s3_client_config).into()
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

async fn prepare_oidc_client(
    oidc_config: &OidcConfiguration,
    url_service: &UrlService,
) -> CoreClient {
    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(oidc_config.server_url.clone()).unwrap(),
        async_client,
    )
    .await
    .unwrap();

    CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(oidc_config.client_id.clone()),
        Some(ClientSecret::new(oidc_config.client_secret.clone())),
    )
    .set_redirect_uri(RedirectUrl::new(url_service.oidc_redirect_uri()).unwrap())
}

async fn prepare_search(
    search_config: &SearchConfiguration,
    db_conn: &DatabaseConnection,
) -> SearchService {
    match search_config {
        SearchConfiguration::Kitsune(config) => {
            GrpcSearchService::connect(&config.index_server, &config.search_servers)
                .await
                .expect("Failed to connect to the search servers")
                .into()
        }
        SearchConfiguration::Meilisearch(_config) => {
            #[cfg(feature = "meilisearch")]
            // To avoid an "unused variable" warning in case the feature is deactivated
            #[allow(clippy::used_underscore_binding)]
            return MeiliSearchService::new(&_config.instance_url, &_config.api_key)
                .await
                .expect("Failed to connect to Meilisearch")
                .into();

            #[cfg(not(feature = "meilisearch"))]
            panic!("Server compiled without Meilisearch compatibility");
        }
        SearchConfiguration::Sql => SqlSearchService::new(db_conn.clone()).into(),
        SearchConfiguration::None => NoopSearchService.into(),
    }
}

#[allow(clippy::too_many_lines)] // TODO: Refactor this method to get under the 100 lines
async fn initialise_state(config: &Configuration, conn: DatabaseConnection) -> Zustand {
    let messaging_hub = prepare_messaging(config).await;
    let status_event_emitter = messaging_hub.emitter("event.status".into());

    let search_service = prepare_search(&config.search, &conn).await;

    let fetcher = Fetcher::builder()
        .db_conn(conn.clone())
        .post_cache(prepare_cache(config, "ACTIVITYPUB-POST"))
        .search_service(search_service.clone())
        .user_cache(prepare_cache(config, "ACTIVITYPUB-USER"))
        .build();

    let webfinger = Webfinger::new(prepare_cache(config, "WEBFINGER"));

    let url_service = UrlService::builder()
        .scheme(config.url.scheme.as_str())
        .domain(config.url.domain.as_str())
        .build();

    let account_service = AccountService::builder()
        .db_conn(conn.clone())
        .url_service(url_service.clone())
        .build();

    let attachment_service = AttachmentService::builder()
        .db_conn(conn.clone())
        .media_proxy_enabled(config.server.media_proxy_enabled)
        .storage_backend(prepare_storage(config))
        .url_service(url_service.clone())
        .build();

    let instance_service = InstanceService::builder()
        .db_conn(conn.clone())
        .name(config.instance.name.as_str())
        .description(config.instance.description.as_str())
        .character_limit(config.instance.character_limit)
        .build();

    let job_service = JobService::builder().db_conn(conn.clone()).build();

    let oidc_service = OptionFuture::from(config.server.oidc.as_ref().map(|oidc_config| async {
        OidcService::builder()
            .client(prepare_oidc_client(oidc_config, &url_service).await)
            .login_state(prepare_cache(config, "OIDC-LOGIN-STATE"))
            .build()
    }))
    .await;

    let oauth2_service = Oauth2Service::builder()
        .db_conn(conn.clone())
        .url_service(url_service.clone())
        .build();

    let post_resolver = PostResolver::new(conn.clone(), fetcher.clone(), webfinger.clone());
    let post_service = PostService::builder()
        .db_conn(conn.clone())
        .instance_service(instance_service.clone())
        .job_service(job_service.clone())
        .post_resolver(post_resolver)
        .search_service(search_service.clone())
        .status_event_emitter(status_event_emitter.clone())
        .url_service(url_service.clone())
        .build();

    let timeline_service = TimelineService::builder().db_conn(conn.clone()).build();

    let user_service = UserService::builder()
        .db_conn(conn.clone())
        .registrations_open(config.instance.registrations_open)
        .url_service(url_service.clone())
        .build();

    #[cfg(feature = "mastodon-api")]
    let mastodon_mapper = kitsune::mapping::MastodonMapper::builder()
        .attachment_service(attachment_service.clone())
        .cache_invalidator(
            status_event_emitter
                .consumer()
                .await
                .expect("Failed to register status event consumer"),
        )
        .db_conn(conn.clone())
        .mastodon_cache(prepare_cache(config, "MASTODON-ENTITY"))
        .url_service(url_service.clone())
        .build()
        .unwrap();

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
            instance: instance_service,
            job: job_service,
            oauth2: oauth2_service,
            oidc: oidc_service,
            search: search_service,
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

    let args: Vec<String> = env::args().take(2).collect();
    if args.len() == 1 {
        eprintln!("Usage: {} <Path to configuration file>", args[0]);
        process::exit(1);
    }

    let config = match Configuration::load(&args[1]) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };
    initialise_logging(&config);

    let conn_opts = ConnectOptions::new(config.database.url.clone())
        .max_connections(config.database.max_connections)
        .clone();
    let conn = kitsune_db::connect(conn_opts)
        .await
        .expect("Failed to connect to database");
    let state = initialise_state(&config, conn).await;

    tokio::spawn(self::http::run(state.clone(), config.server.clone()));
    tokio::spawn(self::job::run_dispatcher(
        state.clone(),
        config.server.job_workers,
    ));

    future::pending::<()>().await;
}
