use super::handle::handle;
use http_body_util::Empty;
use hyper::{body::Bytes, Request, Response};
use kitsune_activitypub::{error::Error, Fetcher};
use kitsune_cache::NoopCache;
use kitsune_config::instance::FederationFilterConfiguration;
use kitsune_core::traits::Fetcher as _;
use kitsune_federation_filter::FederationFilter;
use kitsune_http_client::Client;
use kitsune_search::NoopSearchService;
use kitsune_test::{database_test, language_detection_config};
use kitsune_webfinger::Webfinger;
use std::{convert::Infallible, sync::Arc};
use tower::service_fn;

#[tokio::test]
async fn federation_allow() {
    database_test(|db_pool| async move {
        let builder = Fetcher::builder()
            .db_pool(db_pool)
            .embed_client(None)
            .federation_filter(
                FederationFilter::new(&FederationFilterConfiguration::Allow {
                    domains: vec!["corteximplant.com".into()],
                })
                .unwrap(),
            )
            .search_backend(NoopSearchService)
            .account_cache(Arc::new(NoopCache.into()))
            .post_cache(Arc::new(NoopCache.into()));

        let client = service_fn(
            #[allow(unreachable_code)] // https://github.com/rust-lang/rust/issues/67227
            |_: Request<_>| async {
                panic!("Requested a denied domain") as Result<Response<Empty<Bytes>>, Infallible>
            },
        );
        let client = Client::builder().service(client);
        let fetcher = builder
            .clone()
            .client(client.clone())
            .language_detection_config(language_detection_config())
            .resolver(Arc::new(Webfinger::with_client(
                client,
                Arc::new(NoopCache.into()),
            )))
            .build();

        assert!(matches!(
            *fetcher
                .fetch_post("https://example.com/fakeobject".into())
                .await
                .unwrap_err()
                .downcast_ref()
                .unwrap(),
            Error::BlockedInstance
        ));

        assert!(matches!(
            *fetcher
                .fetch_post("https://other.badstuff.com/otherfake".into())
                .await
                .unwrap_err()
                .downcast_ref()
                .unwrap(),
            Error::BlockedInstance
        ));

        let client = Client::builder().service(service_fn(handle));
        let fetcher = builder
            .clone()
            .client(client.clone())
            .language_detection_config(language_detection_config())
            .resolver(Arc::new(Webfinger::with_client(
                client,
                Arc::new(NoopCache.into()),
            )))
            .build();

        assert!(matches!(
            fetcher
                .fetch_post("https://corteximplant.com/@0x0/109501674056556919".into())
                .await,
            Ok(..)
        ));
    })
    .await;
}

#[tokio::test]
async fn federation_deny() {
    database_test(|db_pool| async move {
        let client = service_fn(
            #[allow(unreachable_code)]
            |_: Request<_>| async {
                panic!("Requested a denied domain") as Result<Response<Empty<Bytes>>, Infallible>
            },
        );
        let client = Client::builder().service(client);

        let fetcher = Fetcher::builder()
            .client(client.clone())
            .db_pool(db_pool)
            .embed_client(None)
            .federation_filter(
                FederationFilter::new(&FederationFilterConfiguration::Deny {
                    domains: vec!["example.com".into(), "*.badstuff.com".into()],
                })
                .unwrap(),
            )
            .language_detection_config(language_detection_config())
            .search_backend(NoopSearchService)
            .resolver(Arc::new(Webfinger::with_client(
                client,
                Arc::new(NoopCache.into()),
            )))
            .account_cache(Arc::new(NoopCache.into()))
            .post_cache(Arc::new(NoopCache.into()))
            .build();

        assert!(matches!(
            fetcher
                .fetch_post("https://example.com/fakeobject".into())
                .await
                .unwrap_err()
                .downcast_ref()
                .unwrap(),
            Error::BlockedInstance
        ));
        assert!(matches!(
            *fetcher
                .fetch_post("https://other.badstuff.com/otherfake".into())
                .await
                .unwrap_err()
                .downcast_ref()
                .unwrap(),
            Error::BlockedInstance
        ));
    })
    .await;
}
