#[macro_use]
extern crate metrics;

#[macro_use]
extern crate tracing;

pub mod consts;
pub mod http;
pub mod oauth2;
pub mod signal;
pub mod state;

use self::{
    oauth2::{OAuth2Service, OAuthEndpoint},
    state::{Service, SessionConfig, Zustand, ZustandInner},
};
use athena::JobQueue;
use color_eyre::eyre;
use kitsune_config::Configuration;
use kitsune_db::PgPool;
use kitsune_email::MailingService;
use kitsune_embed::Client as EmbedClient;
use kitsune_federation::{
    activitypub::PrepareFetcher as PrepareActivityPubFetcher, PrepareFetcher,
};
use kitsune_federation_filter::FederationFilter;
use kitsune_jobs::KitsuneContextRepo;
use kitsune_service::{
    account::AccountService,
    attachment::AttachmentService,
    captcha::CaptchaService,
    custom_emoji::CustomEmojiService,
    instance::InstanceService,
    job::JobService,
    notification::NotificationService,
    post::{PostResolver, PostService},
    prepare,
    search::SearchService,
    timeline::TimelineService,
    user::UserService,
};
use kitsune_url::UrlService;
use kitsune_wasm_mrf::MrfService;

#[cfg(feature = "oidc")]
use {futures_util::future::OptionFuture, kitsune_oidc::OidcService};

#[allow(clippy::too_many_lines)]
pub async fn initialise_state(
    config: &Configuration,
    db_pool: PgPool,
    job_queue: JobQueue<KitsuneContextRepo>,
) -> eyre::Result<Zustand> {
    let url_service = UrlService::builder()
        .domain(config.url.domain.clone())
        .scheme(config.url.scheme.clone())
        .webfinger_domain(config.instance.webfinger_domain.clone())
        .build();

    let attachment_service = AttachmentService::builder()
        .db_pool(db_pool.clone())
        .media_proxy_enabled(config.server.media_proxy_enabled)
        .storage_backend(prepare::storage(&config.storage)?)
        .url_service(url_service.clone())
        .build();

    let federation_filter = FederationFilter::new(&config.instance.federation_filter)
        .map_err(kitsune_error::Error::into_error)?;

    let job_service = JobService::builder().job_queue(job_queue).build();

    let embed_client = config.embed.as_ref().map(|embed_config| {
        EmbedClient::builder()
            .db_pool(db_pool.clone())
            .embed_service(embed_config.service_url.clone())
            .build()
    });

    let search_backend =
        prepare::search(&config.search, config.language_detection, &db_pool).await?;

    let prepare_activitypub_fetcher = PrepareActivityPubFetcher::builder()
        .account_cache(prepare::cache(&config.cache, "ACCOUNT-CACHE").await?)
        .account_resource_cache(prepare::cache(&config.cache, "ACCOUNT-RESOURCE-CACHE").await?)
        .db_pool(db_pool.clone())
        .embed_client(embed_client.clone())
        .federation_filter(federation_filter.clone())
        .language_detection_config(config.language_detection)
        .post_cache(prepare::cache(&config.cache, "POST-CACHE").await?)
        .search_backend(search_backend.clone())
        .build();

    let prepare_fetcher = PrepareFetcher::builder()
        .activitypub(prepare_activitypub_fetcher)
        .build();

    let fetcher = kitsune_federation::prepare_fetcher(prepare_fetcher);

    let account_service = AccountService::builder()
        .attachment_service(attachment_service.clone())
        .db_pool(db_pool.clone())
        .fetcher(fetcher.clone())
        .job_service(job_service.clone())
        .resolver(fetcher.resolver())
        .url_service(url_service.clone())
        .build();

    let captcha_service = CaptchaService::builder()
        .backend(config.captcha.as_ref().map(prepare::captcha))
        .build();

    let custom_emoji_service = CustomEmojiService::builder()
        .attachment_service(attachment_service.clone())
        .db_pool(db_pool.clone())
        .url_service(url_service.clone())
        .build();

    let instance_service = InstanceService::builder()
        .character_limit(config.instance.character_limit)
        .db_pool(db_pool.clone())
        .description(config.instance.description.clone())
        .name(config.instance.name.clone())
        .registrations_open(config.instance.registrations_open)
        .build();

    let mailing_service = MailingService::builder()
        .db_pool(db_pool.clone())
        .sender(
            config
                .email
                .as_ref()
                .map(prepare::mail_sender)
                .transpose()?,
        )
        .url_service(url_service.clone())
        .build();

    #[cfg(feature = "mastodon-api")]
    let mastodon_mapper = kitsune_mastodon::MastodonMapper::builder()
        .attachment_service(attachment_service.clone())
        .db_pool(db_pool.clone())
        .embed_client(embed_client.clone())
        .mastodon_cache(prepare::cache(&config.cache, "MASTODON-CACHE").await?)
        .url_service(url_service.clone())
        .build();

    let mrf_service = MrfService::from_config(&config.mrf).await?;

    let notification_service = NotificationService::builder()
        .db_pool(db_pool.clone())
        .build();

    #[cfg(feature = "oidc")]
    let oidc_service =
        OptionFuture::from(config.server.oidc.as_ref().map(|oidc_config| {
            OidcService::initialise(oidc_config, url_service.oidc_redirect_uri())
        }))
        .await
        .transpose()
        .map_err(kitsune_error::Error::into_error)?;

    let oauth2_service = OAuth2Service::builder()
        .db_pool(db_pool.clone())
        .url_service(url_service.clone())
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
        .language_detection_config(config.language_detection)
        .post_resolver(post_resolver)
        .search_backend(search_backend.clone())
        .url_service(url_service.clone())
        .build();

    let search_service = SearchService::builder()
        .db_pool(db_pool.clone())
        .fetcher(fetcher.clone())
        .search_backend(search_backend)
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

    Ok(ZustandInner {
        db_pool: db_pool.clone(),
        embed_client,
        federation_filter,
        fetcher,
        language_detection_config: config.language_detection,
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
            mrf: mrf_service,
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
