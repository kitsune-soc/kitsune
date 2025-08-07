use kitsune_activitypub::{
    Deliverer as ActivityPubDeliverer, Fetcher as ActivityPubFetcher, InboxResolver,
    deliverer::Service as ActivityPubDelivererService,
};
use kitsune_cache::ArcCache;
use kitsune_config::language_detection::Configuration as LanguageDetectionConfig;
use kitsune_core::traits::{
    Deliverer, Fetcher,
    coerce::{CoerceDeliverer, CoerceFetcher, CoerceResolver},
    resolver::AccountResource,
};
use kitsune_db::{
    PgPool,
    model::{Account, Post},
};
use kitsune_federation_filter::FederationFilter;
use kitsune_search::AnySearchBackend;
use kitsune_service::attachment::AttachmentService;
use kitsune_url::UrlService;
use kitsune_wasm_mrf::MrfService;
use kitsune_webfinger::Webfinger;
use triomphe::Arc;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct PrepareDeliverer {
    attachment_service: AttachmentService,
    db_pool: PgPool,
    federation_filter: FederationFilter,
    http_client: kitsune_http_client::Client,
    mrf_service: MrfService,
    url_service: UrlService,
}

#[derive(TypedBuilder)]
pub struct PrepareFetcher {
    account_cache: ArcCache<str, Account>,
    account_resource_cache: ArcCache<str, AccountResource>,
    db_pool: PgPool,
    embed_client: Option<kitsune_embed::Client>,
    federation_filter: FederationFilter,
    http_client: kitsune_http_client::Client,
    language_detection_config: LanguageDetectionConfig,
    post_cache: ArcCache<str, Post>,
    search_backend: AnySearchBackend,
}

#[inline]
pub(crate) fn prepare_deliverer(prepare: PrepareDeliverer) -> Arc<dyn Deliverer> {
    let core_deliverer = kitsune_activitypub::CoreDeliverer::builder()
        .federation_filter(prepare.federation_filter)
        .http_client(prepare.http_client)
        .mrf_service(prepare.mrf_service)
        .db_pool(prepare.db_pool.clone())
        .build();

    let inbox_resolver = InboxResolver::new(prepare.db_pool.clone());
    let service = ActivityPubDelivererService::builder()
        .attachment(prepare.attachment_service)
        .url(prepare.url_service)
        .build();

    ActivityPubDeliverer::builder()
        .core(core_deliverer)
        .db_pool(prepare.db_pool)
        .inbox_resolver(inbox_resolver)
        .service(service)
        .build()
        .coerce()
}

#[inline]
pub(crate) fn prepare_fetcher(prepare: PrepareFetcher) -> Arc<dyn Fetcher> {
    let webfinger = Webfinger::new(prepare.http_client.clone(), prepare.account_resource_cache);

    ActivityPubFetcher::builder()
        .account_cache(prepare.account_cache)
        .db_pool(prepare.db_pool.clone())
        .embed_client(prepare.embed_client)
        .federation_filter(prepare.federation_filter.clone())
        .http_client(prepare.http_client)
        .language_detection_config(prepare.language_detection_config)
        .post_cache(prepare.post_cache)
        .resolver(Arc::new(webfinger).coerce())
        .search_backend(prepare.search_backend)
        .build()
        .coerce()
}
