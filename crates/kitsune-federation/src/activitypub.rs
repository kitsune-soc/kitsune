use kitsune_activitypub::{
    deliverer::Service as ActivityPubDelivererService, Deliverer as ActivityPubDeliverer,
    Fetcher as ActivityPubFetcher, InboxResolver,
};
use kitsune_cache::ArcCache;
use kitsune_core::traits::resolver::AccountResource;
use kitsune_db::{
    model::{account::Account, post::Post},
    PgPool,
};
use kitsune_federation_filter::FederationFilter;
use kitsune_search::AnySearchBackend;
use kitsune_service::{attachment::AttachmentService, url::UrlService};
use kitsune_webfinger::Webfinger;
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct PrepareDeliverer {
    attachment_service: AttachmentService,
    db_pool: PgPool,
    federation_filter: FederationFilter,
    url_service: UrlService,
}

#[derive(TypedBuilder)]
pub struct PrepareFetcher {
    account_cache: ArcCache<str, Account>,
    account_resource_cache: ArcCache<str, AccountResource>,
    db_pool: PgPool,
    embed_client: Option<kitsune_embed::Client>,
    federation_filter: FederationFilter,
    post_cache: ArcCache<str, Post>,
    search_backend: AnySearchBackend,
}

pub(crate) fn prepare_deliverer(prepare: PrepareDeliverer) -> Arc<ActivityPubDeliverer> {
    let core_deliverer = kitsune_activitypub::CoreDeliverer::builder()
        .federation_filter(prepare.federation_filter)
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
}

pub(crate) fn prepare_fetcher(prepare: PrepareFetcher) -> ActivityPubFetcher<Webfinger> {
    let webfinger = Webfinger::new(prepare.account_resource_cache);

    ActivityPubFetcher::builder()
        .account_cache(prepare.account_cache)
        .db_pool(prepare.db_pool.clone())
        .embed_client(prepare.embed_client)
        .federation_filter(prepare.federation_filter.clone())
        .post_cache(prepare.post_cache)
        .resolver(webfinger)
        .search_backend(prepare.search_backend)
        .build()
}
