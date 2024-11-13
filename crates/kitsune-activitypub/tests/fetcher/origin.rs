use super::handle::handle;
use http::{header::CONTENT_TYPE, uri::PathAndQuery};
use hyper::Request;
use kitsune_activitypub::Fetcher;
use kitsune_cache::NoopCache;
use kitsune_config::instance::FederationFilterConfiguration;
use kitsune_core::traits::{coerce::CoerceResolver, Fetcher as _};
use kitsune_federation_filter::FederationFilter;
use kitsune_http_client::Client;
use kitsune_search::NoopSearchService;
use kitsune_test::{assert_display_eq, database_test, language_detection_config};
use kitsune_webfinger::Webfinger;
use std::convert::Infallible;
use tower::service_fn;
use triomphe::Arc;

#[tokio::test]
async fn check_ap_id_authority() {
    database_test(|db_pool| async move {
        let builder = Fetcher::builder()
            .db_pool(db_pool)
            .embed_client(None)
            .federation_filter(
                FederationFilter::new(&FederationFilterConfiguration::Deny {
                    domains: Vec::new(),
                })
                .unwrap(),
            )
            .search_backend(NoopSearchService)
            .account_cache(Arc::new(NoopCache.into()))
            .post_cache(Arc::new(NoopCache.into()));

        let client = service_fn(|req: Request<_>| {
            assert_ne!(req.uri().host(), Some("corteximplant.com"));
            handle(req)
        });
        let client = Client::builder().service(client);
        let fetcher = builder
            .clone()
            .http_client(client.clone())
            .language_detection_config(language_detection_config())
            .resolver(Arc::new(Webfinger::new(client, Arc::new(NoopCache.into()))).coerce())
            .build();

        // The mock HTTP client ensures that the fetcher doesn't access the correct server
        // so this should return error
        let _ = fetcher
            .fetch_account("https://example.com/users/0x0".into())
            .await
            .unwrap_err();

        let client = service_fn(|req: Request<_>| {
            // Let `fetch_object` fetch `attributedTo`
            if req.uri().path_and_query().map(PathAndQuery::as_str) != Some("/users/0x0") {
                assert_ne!(req.uri().host(), Some("corteximplant.com"));
            }

            handle(req)
        });
        let client = Client::builder().service(client);
        let fetcher = builder
            .clone()
            .http_client(client.clone())
            .language_detection_config(language_detection_config())
            .resolver(Arc::new(Webfinger::new(client, Arc::new(NoopCache.into()))).coerce())
            .build();

        let _ = fetcher
            .fetch_post("https://example.com/@0x0/109501674056556919".into())
            .await
            .unwrap_err();
    })
    .await;
}

#[tokio::test]
async fn check_ap_content_type() {
    database_test(|db_pool| async move {
        let client = service_fn(|req: Request<_>| async {
            let mut res = handle(req).await.unwrap();
            res.headers_mut().remove(CONTENT_TYPE);
            Ok::<_, Infallible>(res)
        });
        let client = Client::builder().service(client);

        let fetcher = Fetcher::builder()
            .http_client(client.clone())
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
            .resolver(Arc::new(Webfinger::new(client, Arc::new(NoopCache.into()))).coerce())
            .account_cache(Arc::new(NoopCache.into()))
            .post_cache(Arc::new(NoopCache.into()))
            .build();

        assert_display_eq!(
            fetcher
                .fetch_post("https://corteximplant.com/users/0x0".into())
                .await
                .unwrap_err(),
            "invalid content-type header in response"
        );
    })
    .await;
}
