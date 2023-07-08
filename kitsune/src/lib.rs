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
        CacheConfiguration, Configuration, JobQueueConfiguration, MessagingConfiguration,
        SearchConfiguration, StorageConfiguration,
    },
    job::KitsuneContextRepo,
    resolve::PostResolver,
    service::{
        account::AccountService,
        attachment::AttachmentService,
        federation_filter::FederationFilterService,
        instance::InstanceService,
        job::JobService,
        oauth2::{OAuth2Service, OAuthEndpoint},
        post::PostService,
        timeline::TimelineService,
        url::UrlService,
        user::UserService,
    },
    state::{EventEmitter, Service, Zustand},
    webfinger::Webfinger,
};
use anyhow::Context;
use athena::JobQueue;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use config::EmailConfiguration;
use kitsune_cache::{ArcCache, InMemoryCache, NoopCache, RedisCache};
use kitsune_db::PgPool;
use kitsune_email::{
    lettre::{AsyncSmtpTransport, Tokio1Executor},
    MailSender,
};
use kitsune_embed::Client as EmbedClient;
use kitsune_messaging::{
    redis::RedisMessagingBackend, tokio_broadcast::TokioBroadcastMessagingBackend, MessagingHub,
};
use kitsune_search::{NoopSearchService, SearchService, SqlSearchService};
use kitsune_storage::{fs::Storage as FsStorage, s3::Storage as S3Storage, Storage};
use serde::{de::DeserializeOwned, Serialize};
use service::mailing::MailingService;
use state::SessionConfig;
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
                let config = deadpool_redis::Config::from_url(redis_config.redis_url.clone());
                config
                    .create_pool(Some(deadpool_redis::Runtime::Tokio1))
                    .unwrap()
            });

            RedisCache::builder()
                .prefix(cache_name)
                .redis_conn(pool.clone())
                .ttl(Duration::from_secs(60)) // TODO: Parameterise this
                .build()
                .expect("[Bug] Failed to build the Redis cache")
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
                .region(Region::new(s3_config.region.to_string()))
                .endpoint_url(s3_config.endpoint_url.as_str())
                .force_path_style(s3_config.force_path_style)
                .credentials_provider(Credentials::from_keys(
                    s3_config.access_key.as_str(),
                    s3_config.secret_access_key.as_str(),
                    None,
                ))
                .build();

            S3Storage::new(s3_config.bucket_name.to_string(), s3_client_config).into()
        }
    }
}

fn prepare_mail_sender(
    config: &EmailConfiguration,
) -> anyhow::Result<MailSender<AsyncSmtpTransport<Tokio1Executor>>> {
    let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(config.host.as_str())?
        .credentials((config.username.as_str(), config.password.as_str()).into())
        .build();

    Ok(MailSender::builder().backend(transport).build())
}

async fn prepare_messaging(config: &Configuration) -> anyhow::Result<MessagingHub> {
    let backend = match config.messaging {
        MessagingConfiguration::InProcess => {
            MessagingHub::new(TokioBroadcastMessagingBackend::default())
        }
        MessagingConfiguration::Redis(ref redis_config) => {
            let redis_messaging_backend = RedisMessagingBackend::new(&redis_config.redis_url)
                .await
                .context("Failed to initialise Redis messaging backend")?;

            MessagingHub::new(redis_messaging_backend)
        }
    };

    Ok(backend)
}

#[cfg(feature = "oidc")]
async fn prepare_oidc_client(
    oidc_config: &OidcConfiguration,
    url_service: &UrlService,
) -> anyhow::Result<CoreClient> {
    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(oidc_config.server_url.to_string()).context("Invalid OIDC issuer URL")?,
        async_client,
    )
    .await
    .context("Couldn't discover the OIDC provider metadata")?;

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(oidc_config.client_id.to_string()),
        Some(ClientSecret::new(oidc_config.client_secret.to_string())),
    )
    .set_redirect_uri(RedirectUrl::new(url_service.oidc_redirect_uri())?);

    Ok(client)
}

#[allow(clippy::unused_async)] // "async" is only unused when none of the more advanced searches are compiled in
async fn prepare_search(
    search_config: &SearchConfiguration,
    db_conn: &PgPool,
) -> anyhow::Result<SearchService> {
    let service = match search_config {
        SearchConfiguration::Kitsune(_config) => {
            #[cfg(not(feature = "kitsune-search"))]
            panic!("Server compiled without Kitsune Search compatibility");

            #[cfg(feature = "kitsune-search")]
            #[allow(clippy::used_underscore_binding)]
            GrpcSearchService::connect(
                &_config.index_server,
                _config.search_servers.iter().map(ToString::to_string),
            )
            .await
            .context("Failed to connect to the search servers")?
            .into()
        }
        SearchConfiguration::Meilisearch(_config) => {
            #[cfg(not(feature = "meilisearch"))]
            panic!("Server compiled without Meilisearch compatibility");

            #[cfg(feature = "meilisearch")]
            #[allow(clippy::used_underscore_binding)]
            MeiliSearchService::new(&_config.instance_url, &_config.api_key)
                .await
                .context("Failed to connect to Meilisearch")?
                .into()
        }
        SearchConfiguration::Sql => SqlSearchService::new(db_conn.clone()).into(),
        SearchConfiguration::None => NoopSearchService.into(),
    };

    Ok(service)
}

pub fn prepare_job_queue(
    db_pool: PgPool,
    config: &JobQueueConfiguration,
) -> anyhow::Result<JobQueue<KitsuneContextRepo>> {
    let context_repo = KitsuneContextRepo::builder().db_pool(db_pool).build();
    let redis_pool = deadpool_redis::Config::from_url(config.redis_url.as_str())
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

    let queue = JobQueue::builder()
        .context_repository(context_repo)
        .queue_name("kitsune-jobs")
        .redis_pool(redis_pool)
        .build();

    Ok(queue)
}

#[allow(clippy::missing_panics_doc, clippy::too_many_lines)] // TODO: Refactor this method to get under the 100 lines
pub async fn initialise_state(
    config: &Configuration,
    conn: PgPool,
    job_queue: JobQueue<KitsuneContextRepo>,
) -> anyhow::Result<Zustand> {
    let messaging_hub = prepare_messaging(config).await?;
    let status_event_emitter = messaging_hub.emitter("event.status".into());

    let search_service = prepare_search(&config.search, &conn).await?;

    let embed_client = config.embed.as_ref().map(|embed_config| {
        EmbedClient::builder()
            .db_pool(conn.clone())
            .embed_service(embed_config.url.clone())
            .build()
    });

    let federation_filter_service =
        FederationFilterService::new(&config.instance.federation_filter)
            .context("Couldn't build the federation filter (check your glob syntax)")?;

    let fetcher = Fetcher::builder()
        .db_conn(conn.clone())
        .embed_client(embed_client.clone())
        .federation_filter(federation_filter_service.clone())
        .post_cache(prepare_cache(config, "ACTIVITYPUB-POST"))
        .search_service(search_service.clone())
        .user_cache(prepare_cache(config, "ACTIVITYPUB-USER"))
        .build();

    let webfinger = Webfinger::new(prepare_cache(config, "WEBFINGER"));

    let job_service = JobService::builder().job_queue(job_queue).build();

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

    let mail_sender = config.email.as_ref().map(prepare_mail_sender).transpose()?;
    let mailing_service = MailingService::builder()
        .sender(mail_sender)
        .url_service(url_service.clone())
        .build();

    #[cfg(feature = "oidc")]
    let oidc_service = OptionFuture::from(config.server.oidc.as_ref().map(|oidc_config| async {
        let service = OidcService::builder()
            .client(prepare_oidc_client(oidc_config, &url_service).await?)
            .login_state(prepare_cache(config, "OIDC-LOGIN-STATE"))
            .build();

        anyhow::Ok(service)
    }))
    .await
    .transpose()?;

    let oauth2_service = OAuth2Service::builder()
        .db_conn(conn.clone())
        .url_service(url_service.clone())
        .build();

    let post_resolver = PostResolver::builder()
        .account(account_service.clone())
        .build();

    let post_service = PostService::builder()
        .db_conn(conn.clone())
        .embed_client(embed_client.clone())
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
        .job_service(job_service.clone())
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
        .embed_client(embed_client.clone())
        .mastodon_cache(prepare_cache(config, "MASTODON-ENTITY"))
        .url_service(url_service.clone())
        .build()
        .expect("[Bug] Failed to initialise Mastodon mapper");

    Ok(Zustand {
        db_conn: conn.clone(),
        embed_client,
        event_emitter: EventEmitter {
            post: status_event_emitter.clone(),
        },
        fetcher,
        #[cfg(feature = "mastodon-api")]
        mastodon_mapper,
        oauth_endpoint: OAuthEndpoint::from(conn),
        service: Service {
            account: account_service,
            federation_filter: federation_filter_service,
            instance: instance_service,
            job: job_service,
            mailing: mailing_service,
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
        session_config: SessionConfig::generate(),
        webfinger,
    })
}
