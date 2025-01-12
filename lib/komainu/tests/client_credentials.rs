use komainu::{extract::ClientCredentials, params::ParamStorage};
use std::borrow::Cow;

const CLIENT_ID: &str = "client_id";
const CLIENT_SECRET: &str = "client_secret";

fn auth_headers() -> http::HeaderMap {
    let encoded = base64_simd::STANDARD.encode_to_string(format!("{CLIENT_ID}:{CLIENT_SECRET}"));
    let header_value =
        http::HeaderValue::from_bytes(format!("Basic {encoded}").as_bytes()).unwrap();

    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::AUTHORIZATION, header_value);

    headers
}

fn auth_body() -> ParamStorage<Cow<'static, str>, Cow<'static, str>> {
    let mut body = ParamStorage::new();
    body.insert("client_id".into(), CLIENT_ID.into());
    body.insert("client_secret".into(), CLIENT_SECRET.into());
    body
}

#[test]
fn basic_auth() {
    let body = ParamStorage::default();
    let headers = auth_headers();

    let client_credentials = ClientCredentials::extract(&headers, &body).unwrap();
    assert_eq!(client_credentials.client_id(), CLIENT_ID);
    assert_eq!(client_credentials.client_secret(), CLIENT_SECRET);
}

#[test]
fn body() {
    let headers = http::HeaderMap::new();
    let body = auth_body();

    let client_credentials = ClientCredentials::extract(&headers, &body).unwrap();
    assert_eq!(client_credentials.client_id(), CLIENT_ID);
    assert_eq!(client_credentials.client_secret(), CLIENT_SECRET);
}

#[test]
fn prefers_basic() {
    let headers = auth_headers();

    let mut body = ParamStorage::new();
    body.insert("client_id".into(), "body_id".into());
    body.insert("client_secret".into(), "body_secret".into());

    let client_credentials = ClientCredentials::extract(&headers, &body).unwrap();
    assert_eq!(client_credentials.client_id(), CLIENT_ID);
    assert_eq!(client_credentials.client_secret(), CLIENT_SECRET);
}

#[test]
fn rejects_missing() {
    let headers = http::HeaderMap::new();
    let body = ParamStorage::new();

    let creds_option = ClientCredentials::extract(&headers, &body);
    assert!(creds_option.is_none());
}
