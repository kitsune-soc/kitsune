use hyper::{Body, Request};
use kitsune_http_client::Client;

#[tokio::test]
async fn basic_request() {
    let mut client = Client::builder().build();
    let req = Request::builder()
        .uri("https://rust-lang.org")
        .body(Body::empty())
        .unwrap();
    let response = client.execute(req).await.unwrap();

    assert!(response.status().is_success());
}
