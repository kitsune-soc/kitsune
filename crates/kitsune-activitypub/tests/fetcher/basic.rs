use self::handle::handle;
use super::MAX_FETCH_DEPTH;
use crate::{
    error::{ApiError, Error},
    service::federation_filter::FederationFilterService,
    webfinger::Webfinger,
    Fetcher,
};
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use http::{header::CONTENT_TYPE, uri::PathAndQuery};
use hyper::{Body, Request, Response, StatusCode, Uri};
use iso8601_timestamp::Timestamp;
use kitsune_cache::NoopCache;
use kitsune_config::instance::FederationFilterConfiguration;
use kitsune_db::{
    model::{account::Account, media_attachment::MediaAttachment},
    schema::{accounts, media_attachments},
};
use kitsune_http_client::Client;
use kitsune_search::NoopSearchService;
use kitsune_test::{build_ap_response, database_test};
use kitsune_type::{
    ap::{
        actor::{Actor, ActorType, PublicKey},
        ap_context, AttributedToField, Object, ObjectType, PUBLIC_IDENTIFIER,
    },
    webfinger::{Link, Resource},
};
use pretty_assertions::assert_eq;
use scoped_futures::ScopedFutureExt;
use std::{
    convert::Infallible,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use tower::service_fn;

mod handle;

#[tokio::test]
#[serial_test::serial]
async fn fetch_actor() {
    database_test(|db_pool| async move {
        let client = Client::builder().service(service_fn(handle));

        let fetcher = Fetcher::builder()
            .client(client.clone())
            .db_pool(db_pool)
            .embed_client(None)
            .federation_filter(
                FederationFilterService::new(&FederationFilterConfiguration::Deny {
                    domains: Vec::new(),
                })
                .unwrap(),
            )
            .search_backend(NoopSearchService)
            .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
            .post_cache(Arc::new(NoopCache.into()))
            .user_cache(Arc::new(NoopCache.into()))
            .build();

        let user = fetcher
            .fetch_actor("https://corteximplant.com/users/0x0".into())
            .await
            .expect("Fetch actor");

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
#[serial_test::serial]
async fn fetch_emoji() {
    database_test(|db_pool| async move {
        let client = Client::builder().service(service_fn(handle));

        let fetcher = Fetcher::builder()
            .client(client.clone())
            .db_pool(db_pool.clone())
            .embed_client(None)
            .federation_filter(
                FederationFilterService::new(&FederationFilterConfiguration::Deny {
                    domains: Vec::new(),
                })
                .unwrap(),
            )
            .search_backend(NoopSearchService)
            .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
            .post_cache(Arc::new(NoopCache.into()))
            .user_cache(Arc::new(NoopCache.into()))
            .build();

        let emoji = fetcher
            .fetch_emoji("https://corteximplant.com/emojis/7952")
            .await
            .expect("Fetch emoji");

        assert_eq!(emoji.shortcode, "Blobhaj");
        assert_eq!(emoji.domain, Some(String::from("corteximplant.com")));

        let media_attachment = db_pool
            .with_connection(|db_conn| {
                media_attachments::table
                    .find(emoji.media_attachment_id)
                    .select(MediaAttachment::as_select())
                    .get_result::<MediaAttachment>(db_conn)
                    .scoped()
            })
            .await
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
#[serial_test::serial]
async fn fetch_note() {
    database_test(|db_pool| async move {
        let client = Client::builder().service(service_fn(handle));

        let fetcher = Fetcher::builder()
            .client(client.clone())
            .db_pool(db_pool.clone())
            .embed_client(None)
            .federation_filter(
                FederationFilterService::new(&FederationFilterConfiguration::Deny {
                    domains: Vec::new(),
                })
                .unwrap(),
            )
            .search_backend(NoopSearchService)
            .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
            .post_cache(Arc::new(NoopCache.into()))
            .user_cache(Arc::new(NoopCache.into()))
            .build();

        let note = fetcher
            .fetch_object("https://corteximplant.com/@0x0/109501674056556919")
            .await
            .expect("Fetch note");
        assert_eq!(
            note.url,
            "https://corteximplant.com/users/0x0/statuses/109501674056556919"
        );

        let author = db_pool
            .with_connection(|db_conn| {
                accounts::table
                    .find(note.account_id)
                    .select(Account::as_select())
                    .get_result::<Account>(db_conn)
                    .scoped()
            })
            .await
            .expect("Get author");

        assert_eq!(author.username, "0x0");
        assert_eq!(author.url, "https://corteximplant.com/users/0x0");
    })
    .await;
}
