#[futures_test::test]
async fn empty_body_accepted() {
    let req = http::Request::new(String::new());
    komainu::Request::read_from(req).await.unwrap();
}

#[futures_test::test]
async fn reject_malformed() {
    let req = http::Request::builder()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(String::from(r#"{"missing": "quote}"#))
        .unwrap();

    let result = komainu::Request::read_from(req).await;
    assert!(result.is_err());
}

#[futures_test::test]
async fn defaults_to_urlencoded() {
    let req = http::Request::new(String::from("hello=world"));
    let parsed = komainu::Request::read_from(req).await.unwrap();
    assert_eq!(parsed.body.get("hello").map(|val| &**val), Some("world"));
}
