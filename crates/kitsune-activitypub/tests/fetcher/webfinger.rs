use super::handle::handle;
use http_body_util::Full;
use hyper::{Request, Response};
use kitsune_activitypub::Fetcher;
use kitsune_cache::NoopCache;
use kitsune_config::instance::FederationFilterConfiguration;
use kitsune_core::traits::{Fetcher as _, coerce::CoerceResolver};
use kitsune_federation_filter::FederationFilter;
use kitsune_http_client::Client;
use kitsune_search::NoopSearchService;
use kitsune_test::{database_test, language_detection_config};
use kitsune_type::webfinger::{Link, Resource};
use kitsune_webfinger::Webfinger;
use pretty_assertions::assert_eq;
use std::convert::Infallible;
use tower::service_fn;
use triomphe::Arc;

#[tokio::test]
async fn fetch_actor_with_custom_acct() {
    database_test(|db_pool| async move {
        let jrd_base = include_bytes!("../../../../test-fixtures/activitypub/0x0_jrd.json");
        let jrd_body = sonic_rs::to_string(&Resource {
            subject: "acct:0x0@joinkitsune.org".into(),
            ..sonic_rs::from_slice(jrd_base).unwrap()
        })
        .unwrap();
        let client = service_fn(move |req: Request<_>| {
            let jrd_body = jrd_body.clone();
            async move {
                match (
                    req.uri().authority().unwrap().as_str(),
                    req.uri().path_and_query().unwrap().as_str(),
                ) {
                    (
                        "corteximplant.com",
                        "/.well-known/webfinger?resource=acct:0x0@corteximplant.com",
                    )
                    | (
                        "joinkitsune.org",
                        "/.well-known/webfinger?resource=acct:0x0@joinkitsune.org",
                    ) => Ok::<_, Infallible>(Response::new(Full::from(jrd_body))),
                    _ => handle(req).await,
                }
            }
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

        let user = fetcher
            .fetch_account("https://corteximplant.com/users/0x0".into())
            .await
            .expect("Fetch actor")
            .unwrap();

        assert_eq!(user.username, "0x0");
        assert_eq!(user.domain, "joinkitsune.org");
        assert_eq!(user.url, "https://corteximplant.com/users/0x0");
    })
    .await;
}

#[tokio::test]
async fn ignore_fake_webfinger_acct() {
    database_test(|db_pool| async move {
        let link = Link {
            rel: "self".to_owned(),
            r#type: Some("application/activity+json".to_owned()),
            href: Some("https://social.whitehouse.gov/users/POTUS".to_owned()),
        };
        let jrd = Resource {
            subject: "acct:POTUS@whitehouse.gov".into(),
            aliases: Vec::new(),
            links: vec![link.clone()],
        };
        let client = service_fn(move |req: Request<_>| {
            let link = link.clone();
            let jrd = jrd.clone();
            async move {
                match (
                    req.uri().authority().unwrap().as_str(),
                    req.uri().path_and_query().unwrap().as_str(),
                ) {
                    (
                        "corteximplant.com",
                        "/.well-known/webfinger?resource=acct:0x0@corteximplant.com",
                    ) => {
                        let fake_jrd = Resource {
                            links: vec![Link {
                                href: Some("https://corteximplant.com/users/0x0".to_owned()),
                                ..link
                            }],
                            ..jrd
                        };
                        let body = sonic_rs::to_string(&fake_jrd).unwrap();
                        Ok::<_, Infallible>(Response::new(Full::from(body)))
                    }
                    (
                        "whitehouse.gov",
                        "/.well-known/webfinger?resource=acct:POTUS@whitehouse.gov",
                    ) => {
                        let body = sonic_rs::to_string(&jrd).unwrap();
                        Ok(Response::new(Full::from(body)))
                    }
                    _ => handle(req).await,
                }
            }
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

        let user = fetcher
            .fetch_account("https://corteximplant.com/users/0x0".into())
            .await
            .expect("Fetch actor")
            .unwrap();

        assert_eq!(user.username, "0x0");
        assert_eq!(user.domain, "corteximplant.com");
        assert_eq!(user.url, "https://corteximplant.com/users/0x0");
    })
    .await;
}
