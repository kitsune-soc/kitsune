use hyper::{Body, Request};
use kitsune_http_client::Client;
use simd_json::{OwnedValue, ValueAccess};

#[tokio::test]
async fn json_request() {
    let client = Client::builder()
        .default_header("Accept", "application/activity+json")
        .unwrap()
        .build();

    let req = Request::builder()
        .uri("https://corteximplant.com/users/0x0")
        .body(Body::empty())
        .unwrap();

    let response = client.execute(req).await.unwrap();
    assert!(response.status().is_success());

    let body: OwnedValue = response.json().await.unwrap();
    assert_eq!(body["preferredUsername"].as_str(), Some("0x0"));
}
