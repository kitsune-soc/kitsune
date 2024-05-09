use super::handle::handle;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_activitypub::Fetcher;
use kitsune_cache::NoopCache;
use kitsune_config::instance::FederationFilterConfiguration;
use kitsune_core::traits::{coerce::CoerceResolver, Fetcher as _};
use kitsune_db::{
    model::{account::Account, media_attachment::MediaAttachment},
    schema::{accounts, media_attachments},
    with_connection_panicky,
};
use kitsune_federation_filter::FederationFilter;
use kitsune_http_client::Client;
use kitsune_search::NoopSearchService;
use kitsune_test::{database_test, language_detection_config};
use kitsune_webfinger::Webfinger;
use pretty_assertions::assert_eq;
use tower::service_fn;
use triomphe::Arc;

#[tokio::test]
async fn fetch_actor() {
    database_test(|db_pool| async move {
        let client = Client::builder().service(service_fn(handle));

        let fetcher = Fetcher::builder()
            .client(client.clone())
            .db_pool(db_pool)
            .embed_client(None)
            .federation_filter(
                FederationFilter::new(&FederationFilterConfiguration::Deny {
                    domains: Vec::new(),
                })
                .unwrap(),
            )
            .language_detection_config(language_detection_config())
            .search_backend(NoopSearchService)
            .resolver(Arc::new(Webfinger::with_client(client, Arc::new(NoopCache.into()))).coerce())
            .account_cache(Arc::new(NoopCache.into()))
            .post_cache(Arc::new(NoopCache.into()))
            .build();

        let user = fetcher
            .fetch_account("https://corteximplant.com/users/0x0".into())
            .await
            .expect("Fetch actor")
            .unwrap();

        assert_eq!(user.username, "0x0");
        assert_eq!(user.domain, "corteximplant.com");
        assert_eq!(user.url, "https://corteximplant.com/users/0x0");
        assert_eq!(
            user.inbox_url,
            Some("https://corteximplant.com/users/0x0/inbox".into())
        );
    })
    .await;
}

#[tokio::test]
async fn fetch_emoji() {
    database_test(|db_pool| async move {
        let client = Client::builder().service(service_fn(handle));

        let fetcher = Fetcher::builder()
            .client(client.clone())
            .db_pool(db_pool.clone())
            .embed_client(None)
            .federation_filter(
                FederationFilter::new(&FederationFilterConfiguration::Deny {
                    domains: Vec::new(),
                })
                .unwrap(),
            )
            .language_detection_config(language_detection_config())
            .search_backend(NoopSearchService)
            .resolver(Arc::new(Webfinger::with_client(client, Arc::new(NoopCache.into()))).coerce())
            .account_cache(Arc::new(NoopCache.into()))
            .post_cache(Arc::new(NoopCache.into()))
            .build();

        let emoji = fetcher
            .fetch_emoji("https://corteximplant.com/emojis/7952")
            .await
            .expect("Fetch emoji")
            .unwrap();

        assert_eq!(emoji.shortcode, "Blobhaj");
        assert_eq!(emoji.domain, Some(String::from("corteximplant.com")));

        let media_attachment =
            with_connection_panicky!(db_pool, |db_conn| {
                media_attachments::table
                    .find(emoji.media_attachment_id)
                    .select(MediaAttachment::as_select())
                    .get_result::<MediaAttachment>(db_conn)
                    .await
            })
            .expect("Get media attachment");

        assert_eq!(
            media_attachment.remote_url,
            Some(String::from(
                "https://corteximplant.com/system/custom_emojis/images/000/007/952/original/33b7f12bd094b815.png"
            ))
        );
    })
    .await;
}

#[tokio::test]
async fn fetch_note() {
    database_test(|db_pool| async move {
        let client = Client::builder().service(service_fn(handle));

        let fetcher = Fetcher::builder()
            .client(client.clone())
            .db_pool(db_pool.clone())
            .embed_client(None)
            .federation_filter(
                FederationFilter::new(&FederationFilterConfiguration::Deny {
                    domains: Vec::new(),
                })
                .unwrap(),
            )
            .language_detection_config(language_detection_config())
            .search_backend(NoopSearchService)
            .resolver(Arc::new(Webfinger::with_client(client, Arc::new(NoopCache.into()))).coerce())
            .account_cache(Arc::new(NoopCache.into()))
            .post_cache(Arc::new(NoopCache.into()))
            .build();

        let note = fetcher
            .fetch_post("https://corteximplant.com/@0x0/109501674056556919".into())
            .await
            .expect("Fetch note")
            .unwrap();

        assert_eq!(
            note.url,
            "https://corteximplant.com/users/0x0/statuses/109501674056556919"
        );

        let author = with_connection_panicky!(db_pool, |db_conn| {
            accounts::table
                .find(note.account_id)
                .select(Account::as_select())
                .get_result::<Account>(db_conn)
                .await
        })
        .expect("Get author");

        assert_eq!(author.username, "0x0");
        assert_eq!(author.url, "https://corteximplant.com/users/0x0");
    })
    .await;
}
