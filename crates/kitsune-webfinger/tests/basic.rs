use hyper::{Body, Request, Response};
use kitsune_cache::NoopCache;
use kitsune_core::traits::Resolver;
use kitsune_http_client::Client;
use kitsune_webfinger::Webfinger;
use pretty_assertions::assert_eq;
use std::{convert::Infallible, sync::Arc};
use tower::service_fn;

#[tokio::test]
async fn basic() {
    let client = service_fn(|req: Request<_>| async move {
        assert_eq!(
            req.uri().path_and_query().unwrap(),
            "/.well-known/webfinger?resource=acct:0x0@corteximplant.com"
        );
        let body = include_str!("../../../test-fixtures/0x0_jrd.json");
        Ok::<_, Infallible>(Response::new(Body::from(body)))
    });
    let client = Client::builder().service(client);

    let webfinger = Webfinger::with_client(client, Arc::new(NoopCache.into()));
    let resource = webfinger
        .resolve_account("0x0", "corteximplant.com")
        .await
        .expect("Failed to fetch resource")
        .unwrap();

    assert_eq!(resource.username, "0x0");
    assert_eq!(resource.domain, "corteximplant.com");
    assert_eq!(resource.uri, "https://corteximplant.com/users/0x0");
}
