#![feature(iter_intersperse)]
#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    forbidden_lint_groups
)]

#[macro_use]
extern crate metrics;

#[macro_use]
extern crate tracing;

pub mod consts;
pub mod error;
pub mod http;
pub mod oauth2;
pub mod state;

use self::{
    oauth2::{OAuth2Service, OAuthEndpoint},
    state::{EventEmitter, Service, SessionConfig, Zustand, ZustandInner},
};
use athena::JobQueue;
use kitsune_config::Configuration;
use kitsune_db::PgPool;
use kitsune_embed::Client as EmbedClient;
use kitsune_federation_filter::FederationFilter;
use kitsune_jobs::KitsuneContextRepo;
use kitsune_service::{
    attachment::AttachmentService, custom_emoji::CustomEmojiService, instance::InstanceService,
    prepare, url::UrlService,
};

#[cfg(feature = "oidc")]
use {futures_util::future::OptionFuture, kitsune_oidc::OidcService};

pub async fn initialise_state(
    config: &Configuration,
    db_pool: PgPool,
    job_queue: JobQueue<KitsuneContextRepo>,
) -> eyre::Result<Zustand> {
    let messaging_hub = prepare::messaging(&config.messaging).await?;
    let status_event_emitter = messaging_hub.emitter("event.status".into());

    let url_service = UrlService::builder()
        .domain(config.url.domain)
        .scheme(config.url.scheme)
        .webfinger_domain(config.instance.webfinger_domain)
        .build();

    let attachment_service = AttachmentService::builder()
        .db_pool(db_pool.clone())
        .media_proxy_enabled(config.server.media_proxy_enabled)
        .storage_backend(prepare::storage(&config.storage)?)
        .url_service(url_service.clone())
        .build();

    let embed_client = config.embed.as_ref().map(|embed_config| {
        EmbedClient::builder()
            .db_pool(db_pool.clone())
            .embed_service(embed_config.service_url)
            .build()
    });

    let custom_emoji_service = CustomEmojiService::builder()
        .attachment_service(attachment_service.clone())
        .db_pool(db_pool.clone())
        .url_service(url_service.clone())
        .build();

    let instance_service = InstanceService::builder()
        .character_limit(config.instance.character_limit)
        .db_pool(db_pool.clone())
        .description(config.instance.description)
        .name(config.instance.name)
        .registrations_open(config.instance.registrations_open)
        .build();

    #[cfg(feature = "oidc")]
    let oidc_service =
        OptionFuture::from(config.server.oidc.as_ref().map(|oidc_config| {
            OidcService::initialise(oidc_config, url_service.oidc_redirect_uri())
        }))
        .await
        .transpose()?;

    let oauth2_service = OAuth2Service::builder()
        .db_pool(db_pool.clone())
        .url_service(url_service.clone())
        .build();

    #[cfg(feature = "mastodon-api")]
    let mastodon_mapper = kitsune_mastodon::MastodonMapper::builder()
        .attachment_service(attachment_service.clone())
        .cache_invalidator(
            status_event_emitter
                .consumer()
                .await
                .expect("Failed to register status event consumer"),
        )
        .db_pool(db_pool.clone())
        .embed_client(embed_client.clone())
        .mastodon_cache()
        .url_service(url_service.clone())
        .build()
        .unwrap();

    Ok(ZustandInner {
        db_pool: db_pool.clone(),
        embed_client,
        event_emitter: EventEmitter {
            post: status_event_emitter.clone(),
        },
        federation_filter: FederationFilter::new(&config.instance.federation_filter)?,
        #[cfg(feature = "mastodon-api")]
        mastodon_mapper,
        oauth2: oauth2_service,
        oauth_endpoint: OAuthEndpoint::from(db_pool),
        #[cfg(feature = "oidc")]
        oidc: oidc_service,
        service: Service {
            account: account_service,
            attachment: attachment_service,
            captcha: captcha_service,
            custom_emoji: custom_emoji_service,
            job: job_service,
            mailing: mailing_service,
            notification: notification_service,
            post: post_service,
            instance: instance_service,
            search: search_service,
            timeline: timeline_service,
            url: url_service,
            user: user_service,
        },
        session_config: SessionConfig::generate(),
    }
    .into())
}
