#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    forbidden_lint_groups
)]

pub mod error;
pub mod event;
pub mod traits;

/*
use self::{
    activitypub::Fetcher,
    job::KitsuneContextRepo,
    resolve::PostResolver,
    service::{
        account::AccountService, attachment::AttachmentService, captcha::CaptchaService,
        custom_emoji::CustomEmojiService, federation_filter::FederationFilterService,
        instance::InstanceService, job::JobService, mailing::MailingService,
        notification::NotificationService, post::PostService, search::SearchService,
        timeline::TimelineService, url::UrlService, user::UserService,
    },
    state::{EventEmitter, Service, State},
    webfinger::Webfinger,
};
use athena::JobQueue;
use eyre::Context;
use kitsune_cache::{ArcCache, InMemoryCache, NoopCache, RedisCache};
use kitsune_captcha::{hcaptcha::Captcha as HCaptcha, mcaptcha::Captcha as MCaptcha, AnyCaptcha};
use kitsune_config::{cache, captcha, email, messaging, search, storage, Configuration};
use kitsune_db::PgPool;
use kitsune_email::{
    lettre::{message::Mailbox, AsyncSmtpTransport, Tokio1Executor},
    MailSender,
};
use kitsune_embed::Client as EmbedClient;
use kitsune_messaging::{
    redis::RedisMessagingBackend, tokio_broadcast::TokioBroadcastMessagingBackend, MessagingHub,
};
use kitsune_search::{AnySearchBackend, NoopSearchService, SqlSearchService};
use kitsune_storage::{fs::Storage as FsStorage, s3::Storage as S3Storage, AnyStorageBackend};
use rusty_s3::{Bucket as S3Bucket, Credentials as S3Credentials};
use serde::{de::DeserializeOwned, Serialize};

use std::{
    fmt::Display,
    str::FromStr,
    sync::{Arc, OnceLock},
    time::Duration,
};

#[cfg(feature = "meilisearch")]
use kitsune_search::MeiliSearchService;

pub fn prepare_cache<K, V>(config: &Configuration, cache_name: &str) -> ArcCache<K, V>
where
    K: Display + Send + Sync + ?Sized + 'static,
    V: Clone + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    let cache = match config.cache {
        cache::Configuration::InMemory => InMemoryCache::new(100, Duration::from_secs(60)).into(), // TODO: Parameterise this
        cache::Configuration::None => NoopCache.into(),
        cache::Configuration::Redis(ref redis_config) => {
            static REDIS_POOL: OnceLock<deadpool_redis::Pool> = OnceLock::new();

            let pool = REDIS_POOL.get_or_init(|| {
                let config = deadpool_redis::Config::from_url(redis_config.url.clone());
                config
                    .create_pool(Some(deadpool_redis::Runtime::Tokio1))
                    .unwrap()
            });

            RedisCache::builder()
                .prefix(cache_name)
                .redis_conn(pool.clone())
                .ttl(Duration::from_secs(60)) // TODO: Parameterise this
                .build()
                .into()
        }
    };

    Arc::new(cache)
}

fn prepare_captcha(config: &captcha::Configuration) -> AnyCaptcha {
    match config {
        captcha::Configuration::HCaptcha(config) => HCaptcha::builder()
            .verify_url(config.verify_url.to_string())
            .site_key(config.site_key.to_string())
            .secret_key(config.secret_key.to_string())
            .build()
            .into(),
        captcha::Configuration::MCaptcha(config) => MCaptcha::builder()
            .widget_link(config.widget_link.to_string())
            .verify_url(config.verify_url.to_string())
            .site_key(config.site_key.to_string())
            .secret_key(config.secret_key.to_string())
            .build()
            .into(),
    }
}

fn prepare_mail_sender(
    config: &email::Configuration,
) -> eyre::Result<MailSender<AsyncSmtpTransport<Tokio1Executor>>> {
    let transport_builder = if config.starttls {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(config.host.as_str())?
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::relay(config.host.as_str())?
    };

    let transport = transport_builder
        .credentials((config.username.as_str(), config.password.as_str()).into())
        .build();

    Ok(MailSender::builder()
        .backend(transport)
        .from_mailbox(Mailbox::from_str(config.from_address.as_str())?)
        .build())
}

async fn prepare_messaging(config: &Configuration) -> eyre::Result<MessagingHub> {
    let backend = match config.messaging {
        messaging::Configuration::InProcess => {
            MessagingHub::new(TokioBroadcastMessagingBackend::default())
        }
        messaging::Configuration::Redis(ref redis_config) => {
            let redis_messaging_backend = RedisMessagingBackend::new(&redis_config.url)
                .await
                .context("Failed to initialise Redis messaging backend")?;

            MessagingHub::new(redis_messaging_backend)
        }
    };

    Ok(backend)
}

#[allow(clippy::unused_async)] // "async" is only unused when none of the more advanced searches are compiled in
async fn prepare_search(
    search_config: &search::Configuration,
    db_pool: &PgPool,
) -> eyre::Result<AnySearchBackend> {
    let service = match search_config {
        search::Configuration::Meilisearch(_config) => {
            #[cfg(not(feature = "meilisearch"))]
            panic!("Server compiled without Meilisearch compatibility");

            #[cfg(feature = "meilisearch")]
            #[allow(clippy::used_underscore_binding)]
            MeiliSearchService::new(&_config.instance_url, &_config.api_key)
                .await
                .context("Failed to connect to Meilisearch")?
                .into()
        }
        search::Configuration::Sql => SqlSearchService::new(db_pool.clone()).into(),
        search::Configuration::None => NoopSearchService.into(),
    };

    Ok(service)
}

#[allow(clippy::too_many_lines)] // TODO: Refactor to get under 100 lines
pub async fn prepare_state(
    config: &Configuration,
    db_pool: PgPool,
    job_queue: JobQueue<KitsuneContextRepo>,
) -> eyre::Result<State> {
    let messaging_hub = prepare_messaging(config).await?;
    let status_event_emitter = messaging_hub.emitter("event.status".into());

    let search_backend = prepare_search(&config.search, &db_pool).await?;

    let embed_client = config.embed.as_ref().map(|embed_config| {
        EmbedClient::builder()
            .db_pool(db_pool.clone())
            .embed_service(embed_config.service_url.clone())
            .build()
    });

    let federation_filter_service =
        FederationFilterService::new(&config.instance.federation_filter)
            .context("Couldn't build the federation filter (check your glob syntax)")?;

    let webfinger = Webfinger::new(prepare_cache(config, "WEBFINGER"));

    let fetcher = Fetcher::builder()
        .db_pool(db_pool.clone())
        .embed_client(embed_client.clone())
        .federation_filter(federation_filter_service.clone())
        .post_cache(prepare_cache(config, "ACTIVITYPUB-POST"))
        .webfinger(webfinger.clone())
        .search_backend(search_backend.clone())
        .user_cache(prepare_cache(config, "ACTIVITYPUB-USER"))
        .build();

    let search_service = SearchService::builder()
        .db_pool(db_pool.clone())
        .fetcher(fetcher.clone())
        .search_backend(search_backend.clone())
        .build();

    let job_service = JobService::builder().job_queue(job_queue).build();

    let url_service = UrlService::builder()
        .scheme(config.url.scheme.as_str())
        .domain(config.url.domain.as_str())
        .webfinger_domain(config.instance.webfinger_domain.clone())
        .build();

    let attachment_service = AttachmentService::builder()
        .db_pool(db_pool.clone())
        .media_proxy_enabled(config.server.media_proxy_enabled)
        .storage_backend(prepare_storage(config)?)
        .url_service(url_service.clone())
        .build();

    let account_service = AccountService::builder()
        .attachment_service(attachment_service.clone())
        .db_pool(db_pool.clone())
        .fetcher(fetcher.clone())
        .job_service(job_service.clone())
        .url_service(url_service.clone())
        .webfinger(webfinger.clone())
        .build();

    let captcha_backend = config.captcha.as_ref().map(prepare_captcha);
    let captcha_service = CaptchaService::builder().backend(captcha_backend).build();

    let custom_emoji_service = CustomEmojiService::builder()
        .attachment_service(attachment_service.clone())
        .db_pool(db_pool.clone())
        .url_service(url_service.clone())
        .build();

    let instance_service = InstanceService::builder()
        .db_pool(db_pool.clone())
        .name(config.instance.name.as_str())
        .description(config.instance.description.as_str())
        .character_limit(config.instance.character_limit)
        .registrations_open(config.instance.registrations_open)
        .build();

    let mail_sender = config.email.as_ref().map(prepare_mail_sender).transpose()?;
    let mailing_service = MailingService::builder()
        .sender(mail_sender)
        .url_service(url_service.clone())
        .build();

    let notification_service = NotificationService::builder()
        .db_pool(db_pool.clone())
        .build();

    let post_resolver = PostResolver::builder()
        .account(account_service.clone())
        .custom_emoji(custom_emoji_service.clone())
        .build();

    let post_service = PostService::builder()
        .db_pool(db_pool.clone())
        .embed_client(embed_client.clone())
        .instance_service(instance_service.clone())
        .job_service(job_service.clone())
        .post_resolver(post_resolver)
        .search_backend(search_backend)
        .status_event_emitter(status_event_emitter.clone())
        .url_service(url_service.clone())
        .build();

    let timeline_service = TimelineService::builder().db_pool(db_pool.clone()).build();

    let user_service = UserService::builder()
        .allow_non_ascii_usernames(config.instance.allow_non_ascii_usernames)
        .captcha_service(captcha_service.clone())
        .db_pool(db_pool.clone())
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
        .db_pool(db_pool.clone())
        .embed_client(embed_client.clone())
        .mastodon_cache(prepare_cache(config, "MASTODON-ENTITY"))
        .url_service(url_service.clone())
        .build()
        .expect("[Bug] Failed to initialise Mastodon mapper");

    Ok(State {
        db_pool: db_pool.clone(),
        embed_client,
        event_emitter: EventEmitter {
            post: status_event_emitter.clone(),
        },
        fetcher,
        #[cfg(feature = "mastodon-api")]
        mastodon_mapper,
        service: Service {
            account: account_service,
            captcha: captcha_service,
            custom_emoji: custom_emoji_service,
            federation_filter: federation_filter_service,
            instance: instance_service,
            job: job_service,
            mailing: mailing_service,
            notification: notification_service,
            search: search_service,
            post: post_service,
            timeline: timeline_service,
            attachment: attachment_service,
            url: url_service,
            user: user_service,
        },
        webfinger,
    })
}

*/
