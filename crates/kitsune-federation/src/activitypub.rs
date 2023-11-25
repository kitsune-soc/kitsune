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
pub struct PrepareActivityPub {
    account_cache: ArcCache<str, Account>,
    account_resource_cache: ArcCache<str, AccountResource>,
    attachment_service: AttachmentService,
    db_pool: PgPool,
    embed_client: Option<kitsune_embed::Client>,
    federation_filter: FederationFilter,
    post_cache: ArcCache<str, Post>,
    search_backend: AnySearchBackend,
    url_service: UrlService,
}

pub fn prepare_activitypub(
    prepare_activitypub: PrepareActivityPub,
) -> (ActivityPubFetcher<Webfinger>, Arc<ActivityPubDeliverer>) {
    let webfinger = Webfinger::new(prepare_activitypub.account_resource_cache);
    let fetcher = ActivityPubFetcher::builder()
        .account_cache(prepare_activitypub.account_cache)
        .db_pool(prepare_activitypub.db_pool.clone())
        .embed_client(prepare_activitypub.embed_client)
        .federation_filter(prepare_activitypub.federation_filter.clone())
        .post_cache(prepare_activitypub.post_cache)
        .resolver(webfinger)
        .search_backend(prepare_activitypub.search_backend)
        .build();

    let core_deliverer = kitsune_activitypub::CoreDeliverer::builder()
        .federation_filter(prepare_activitypub.federation_filter)
        .build();

    let inbox_resolver = InboxResolver::new(prepare_activitypub.db_pool.clone());
    let service = ActivityPubDelivererService::builder()
        .attachment(prepare_activitypub.attachment_service)
        .url(prepare_activitypub.url_service)
        .build();

    let deliverer = ActivityPubDeliverer::builder()
        .core(core_deliverer)
        .db_pool(prepare_activitypub.db_pool)
        .inbox_resolver(inbox_resolver)
        .service(service)
        .build();

    (fetcher, deliverer)
}
