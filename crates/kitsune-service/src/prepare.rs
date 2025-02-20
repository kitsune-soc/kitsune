use eyre::WrapErr;
use fred::{
    clients::Pool as RedisPool, interfaces::ClientLike, types::config::Config as RedisConfig,
};
use kitsune_cache::{ArcCache, InMemoryCache, NoopCache, RedisCache};
use kitsune_captcha::{hcaptcha::Captcha as HCaptcha, mcaptcha::Captcha as MCaptcha, AnyCaptcha};
use kitsune_config::{cache, captcha, email, language_detection, search, storage};
use kitsune_db::PgPool;
use kitsune_email::{
    lettre::{message::Mailbox, AsyncSmtpTransport, Tokio1Executor},
    MailSender,
};
use kitsune_search::{AnySearchBackend, NoopSearchService, SqlSearchService};
use kitsune_storage::{fs::Storage as FsStorage, s3::Storage as S3Storage, AnyStorageBackend};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Display, str::FromStr, time::Duration};
use tokio::sync::OnceCell;
use triomphe::Arc;

pub async fn cache<K, V>(
    config: &cache::Configuration,
    cache_name: &str,
) -> eyre::Result<ArcCache<K, V>>
where
    K: Display + Send + Sync + ?Sized + 'static,
    V: Clone + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    let cache = match config {
        cache::Configuration::InMemory => InMemoryCache::new(100, Duration::from_secs(60)).into(), // TODO: Parameterise this
        cache::Configuration::None => NoopCache.into(),
        cache::Configuration::Redis(redis_config) => {
            static REDIS_POOL: OnceCell<RedisPool> = OnceCell::const_new();

            let pool = REDIS_POOL
                .get_or_try_init(|| async {
                    let config = RedisConfig::from_url(redis_config.url.as_str())?;
                    let pool = RedisPool::new(config, None, None, None, 10)?;
                    pool.init().await?;

                    eyre::Ok(pool)
                })
                .await?;

            RedisCache::builder()
                .conn_pool(pool.clone())
                .prefix(cache_name)
                .ttl(Duration::from_secs(60)) // TODO: Parameterise this
                .build()
                .into()
        }
    };

    Ok(Arc::new(cache))
}

#[must_use]
pub fn captcha(client: kitsune_http_client::Client, config: &captcha::Configuration) -> AnyCaptcha {
    match config {
        captcha::Configuration::HCaptcha(config) => HCaptcha::builder()
            .verify_url(config.verify_url.to_string())
            .site_key(config.site_key.to_string())
            .secret_key(config.secret_key.to_string())
            .http_client(client)
            .build()
            .into(),
        captcha::Configuration::MCaptcha(config) => MCaptcha::builder()
            .widget_link(config.widget_link.to_string())
            .verify_url(config.verify_url.to_string())
            .site_key(config.site_key.to_string())
            .secret_key(config.secret_key.to_string())
            .http_client(client)
            .build()
            .into(),
    }
}

pub fn storage(config: &storage::Configuration) -> eyre::Result<AnyStorageBackend> {
    let storage = match config {
        storage::Configuration::Fs(fs_config) => {
            FsStorage::new(fs_config.upload_dir.as_str().into()).into()
        }
        storage::Configuration::S3(s3_config) => {
            let path_style = if s3_config.force_path_style {
                rusty_s3::UrlStyle::Path
            } else {
                rusty_s3::UrlStyle::VirtualHost
            };

            let s3_credentials = rusty_s3::Credentials::new(
                s3_config.access_key.as_str(),
                s3_config.secret_access_key.as_str(),
            );
            let s3_bucket = rusty_s3::Bucket::new(
                s3_config.endpoint_url.parse()?,
                path_style,
                s3_config.bucket_name.to_string(),
                s3_config.region.to_string(),
            )?;

            S3Storage::new(s3_bucket, s3_credentials).into()
        }
    };

    Ok(storage)
}

pub fn mail_sender(
    config: &email::Configuration,
) -> eyre::Result<MailSender<AsyncSmtpTransport<Tokio1Executor>>> {
    let transport_builder = if config.starttls {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(config.host.as_str())
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::relay(config.host.as_str())
    }?;

    let transport = transport_builder
        .credentials((config.username.as_str(), config.password.as_str()).into())
        .build();

    Ok(MailSender::builder()
        .backend(transport)
        .from_mailbox(Mailbox::from_str(config.from_address.as_str())?)
        .build())
}

#[allow(clippy::unused_async)] // "async" is only unused when none of the more advanced searches are compiled in
pub async fn search(
    search_config: &search::Configuration,
    language_detection_config: language_detection::Configuration,
    db_pool: &PgPool,
) -> eyre::Result<AnySearchBackend> {
    let service = match search_config {
        search::Configuration::Meilisearch(config) => {
            kitsune_search::MeiliSearchService::new(&config.instance_url, &config.api_key)
                .await
                .map_err(kitsune_error::Error::into_error)
                .wrap_err("Failed to connect to Meilisearch")?
                .into()
        }
        search::Configuration::Sql => SqlSearchService::builder()
            .db_pool(db_pool.clone())
            .language_detection_config(language_detection_config)
            .build()
            .into(),
        search::Configuration::None => NoopSearchService.into(),
    };

    Ok(service)
}
