use http_body_util::Full;
use hyper::{Request, Response, StatusCode, body::Bytes};
use kitsune_cache::NoopCache;
use kitsune_core::traits::Resolver;
use kitsune_http_client::Client;
use kitsune_type::webfinger::Resource;
use kitsune_webfinger::{MAX_JRD_REDIRECTS, Webfinger};
use pretty_assertions::assert_eq;
use std::convert::Infallible;
use tower::service_fn;
use triomphe::Arc;

#[tokio::test]
async fn follow_jrd_redirect() {
    let base = include_bytes!("../../../test-fixtures/activitypub/0x0_jrd.json");
    let body = sonic_rs::to_string(&Resource {
        subject: "acct:0x0@joinkitsune.org".into(),
        ..sonic_rs::from_slice(base).unwrap()
    })
    .unwrap();

    let client = service_fn(move |req: Request<_>| {
        let body = body.clone();
        async move {
            match (
                req.uri().authority().unwrap().as_str(),
                req.uri().path_and_query().unwrap().as_str(),
            ) {
                (
                    "corteximplant.com",
                    "/.well-known/webfinger?resource=acct:0x0@corteximplant.com",
                )
                | ("joinkitsune.org", "/.well-known/webfinger?resource=acct:0x0@joinkitsune.org") => {
                    Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(body))))
                }
                _ => panic!("HTTP client hit unexpected route: {}", req.uri()),
            }
        }
    });

    let client = Client::builder().service(client);

    let webfinger = Webfinger::new(client, Arc::new(NoopCache.into()));
    let resource = webfinger
        .resolve_account("0x0", "corteximplant.com")
        .await
        .expect("Failed to fetch resource")
        .unwrap();

    assert_eq!(resource.username, "0x0");
    assert_eq!(resource.domain, "joinkitsune.org");
    assert_eq!(resource.uri, "https://corteximplant.com/users/0x0");
}

#[tokio::test]
async fn reject_fake_jrd_redirect() {
    let client = service_fn(|req: Request<_>| async move {
        match (
            req.uri().authority().unwrap().as_str(),
            req.uri().path_and_query().unwrap().as_str(),
        ) {
            ("corteximplant.com", "/.well-known/webfinger?resource=acct:0x0@corteximplant.com") => {
                let base = include_bytes!("../../../test-fixtures/activitypub/0x0_jrd.json");
                let body = sonic_rs::to_string(&Resource {
                    subject: "acct:0x0@whitehouse.gov".into(),
                    ..sonic_rs::from_slice(base).unwrap()
                })
                .unwrap();
                Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(body))))
            }
            ("whitehouse.gov", "/.well-known/webfinger?resource=acct:0x0@whitehouse.gov") => {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::default())
                    .unwrap())
            }
            _ => panic!("HTTP client hit unexpected route: {}", req.uri()),
        }
    });
    let client = Client::builder().service(client);

    let webfinger = Webfinger::new(client, Arc::new(NoopCache.into()));
    let resource = webfinger
        .resolve_account("0x0", "corteximplant.com")
        .await
        .expect("Failed to fetch resource");

    assert!(resource.is_none(), "resource = {resource:?}");
}

#[tokio::test]
async fn reject_unbounded_number_of_jrd_redirects() {
    let client = service_fn(|req: Request<_>| async move {
        let Some(count) = req
            .uri()
            .path_and_query()
            .unwrap()
            .as_str()
            .strip_prefix("/.well-known/webfinger?resource=acct:0x")
            .and_then(|suffix| suffix.strip_suffix("@corteximplant.com"))
            .and_then(|count| u32::from_str_radix(count, 16).ok())
        else {
            panic!(
                "HTTP client hit unexpected route: {}",
                req.uri().path_and_query().unwrap()
            );
        };
        assert!(count <= MAX_JRD_REDIRECTS);
        let base = include_bytes!("../../../test-fixtures/activitypub/0x0_jrd.json");
        let body = sonic_rs::to_string(&Resource {
            subject: format!("acct:0x{:x}@corteximplant.com", count + 1),
            ..sonic_rs::from_slice(base).unwrap()
        })
        .unwrap();
        Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(body))))
    });
    let client = Client::builder().service(client);

    let webfinger = Webfinger::new(client, Arc::new(NoopCache.into()));
    let resource = webfinger
        .resolve_account("0x0", "corteximplant.com")
        .await
        .expect("Failed to fetch resource");

    assert!(resource.is_none(), "resource = {resource:?}");
}
