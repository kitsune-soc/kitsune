use core::convert::Infallible;
use hyper::{Body, Request, Response};
use kitsune_http_client::Client;
use tower::service_fn;

#[tokio::test]
async fn basic_request() {
    let client = service_fn(|req: Request<_>| async move {
        assert_eq!(req.uri().path_and_query().unwrap(), "/path");
        Ok::<_, Infallible>(Response::new(Body::empty()))
    });
    let client = Client::builder().service(client);

    let req = Request::builder()
        .uri("https://example.com/path")
        .body(Body::empty())
        .unwrap();
    let response = client.execute(req).await.unwrap();

    assert!(response.status().is_success());
}
