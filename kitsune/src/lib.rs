#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    forbidden_lint_groups
)]

#[cfg(feature = "metrics")]
#[macro_use]
extern crate metrics;

#[macro_use]
extern crate tracing;

#[cfg(test)]
mod test;

pub mod activitypub;
pub mod blocking;
pub mod config;
pub mod consts;
pub mod error;
pub mod event;
pub mod http;
pub mod job;
pub mod mapping;
pub mod resolve;
pub mod sanitize;
pub mod service;
pub mod state;
pub mod util;
pub mod webfinger;

use self::{
    activitypub::Fetcher,
    config::{
        CacheConfiguration, Configuration, MessagingConfiguration, SearchConfiguration,
        StorageConfiguration,
    },
    resolve::PostResolver,
    service::{
        account::AccountService,
        attachment::AttachmentService,
        federation_filter::FederationFilterService,
        instance::InstanceService,
        job::JobService,
        oauth2::{Oauth2Service, OauthEndpoint},
        post::PostService,
        timeline::TimelineService,
        url::UrlService,
        user::UserService,
    },
    state::{EventEmitter, Service, Zustand},
    webfinger::Webfinger,
};
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use kitsune_cache::{ArcCache, InMemoryCache, NoopCache, RedisCache};
use kitsune_db::PgPool;
use kitsune_messaging::{
    redis::RedisMessagingBackend, tokio_broadcast::TokioBroadcastMessagingBackend, MessagingHub,
};
use kitsune_search::{NoopSearchService, SearchService, SqlSearchService};
use kitsune_storage::{fs::Storage as FsStorage, s3::Storage as S3Storage, Storage};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::Display,
    sync::{Arc, OnceLock},
    time::Duration,
};

#[cfg(feature = "kitsune-search")]
use kitsune_search::GrpcSearchService;

#[cfg(feature = "meilisearch")]
use kitsune_search::MeiliSearchService;

#[cfg(feature = "oidc")]
use {
    self::{
        config::OidcConfiguration,
        service::oidc::{async_client, OidcService},
    },
    futures_util::future::OptionFuture,
    openidconnect::{
        core::{CoreClient, CoreProviderMetadata},
        ClientId, ClientSecret, IssuerUrl, RedirectUrl,
    },
};

fn prepare_cache<K, V>(config: &Configuration, cache_name: &str) -> ArcCache<K, V>
where
    K: Display + Send + Sync + ?Sized + 'static,
    V: Clone + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    let cache = match config.cache {
        CacheConfiguration::InMemory => InMemoryCache::new(100, Duration::from_secs(60)).into(), // TODO: Parameterise this
        CacheConfiguration::None => NoopCache.into(),
        CacheConfiguration::Redis(ref redis_config) => {
            static REDIS_POOL: OnceLock<deadpool_redis::Pool> = OnceLock::new();

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

#[cfg(feature = "oidc")]
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

#[allow(clippy::unused_async)] // "async" is only unused when none of the more advanced searches are compiled in
async fn prepare_search(search_config: &SearchConfiguration, db_conn: &PgPool) -> SearchService {
    match search_config {
        SearchConfiguration::Kitsune(_config) => {
            #[cfg(feature = "kitsune-search")]
            #[allow(clippy::used_underscore_binding)]
            return GrpcSearchService::connect(&_config.index_server, &_config.search_servers)
                .await
                .expect("Failed to connect to the search servers")
                .into();

            #[cfg(not(feature = "kitsune-search"))]
            panic!("Server compiled without Kitsune Search compatibility");
        }
        SearchConfiguration::Meilisearch(_config) => {
            #[cfg(feature = "meilisearch")]
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

#[allow(clippy::missing_panics_doc, clippy::too_many_lines)] // TODO: Refactor this method to get under the 100 lines
pub async fn initialise_state(config: &Configuration, conn: PgPool) -> Zustand {
    let messaging_hub = prepare_messaging(config).await;
    let status_event_emitter = messaging_hub.emitter("event.status".into());

    let search_service = prepare_search(&config.search, &conn).await;
    let federation_filter_service =
        FederationFilterService::new(&config.instance.federation_filter).unwrap();

    let fetcher = Fetcher::builder()
        .db_conn(conn.clone())
        .federation_filter(federation_filter_service.clone())
        .post_cache(prepare_cache(config, "ACTIVITYPUB-POST"))
        .search_service(search_service.clone())
        .user_cache(prepare_cache(config, "ACTIVITYPUB-USER"))
        .build();

    let webfinger = Webfinger::new(prepare_cache(config, "WEBFINGER"));

    let job_service = JobService::builder().db_conn(conn.clone()).build();

    let url_service = UrlService::builder()
        .scheme(config.url.scheme.as_str())
        .domain(config.url.domain.as_str())
        .build();

    let attachment_service = AttachmentService::builder()
        .db_conn(conn.clone())
        .media_proxy_enabled(config.server.media_proxy_enabled)
        .storage_backend(prepare_storage(config))
        .url_service(url_service.clone())
        .build();

    let account_service = AccountService::builder()
        .attachment_service(attachment_service.clone())
        .db_conn(conn.clone())
        .fetcher(fetcher.clone())
        .job_service(job_service.clone())
        .url_service(url_service.clone())
        .webfinger(webfinger.clone())
        .build();

    let instance_service = InstanceService::builder()
        .db_conn(conn.clone())
        .name(config.instance.name.as_str())
        .description(config.instance.description.as_str())
        .character_limit(config.instance.character_limit)
        .build();

    #[cfg(feature = "oidc")]
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

    let post_resolver = PostResolver::builder()
        .account(account_service.clone())
        .build();

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
    let mastodon_mapper = self::mapping::MastodonMapper::builder()
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
        db_conn: conn.clone(),
        event_emitter: EventEmitter {
            post: status_event_emitter.clone(),
        },
        fetcher,
        #[cfg(feature = "mastodon-api")]
        mastodon_mapper,
        oauth_endpoint: OauthEndpoint::from(conn),
        service: Service {
            account: account_service,
            federation_filter: federation_filter_service,
            instance: instance_service,
            job: job_service,
            oauth2: oauth2_service,
            #[cfg(feature = "oidc")]
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
