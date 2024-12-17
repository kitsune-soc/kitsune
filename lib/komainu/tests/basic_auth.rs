use komainu::extractor::BasicAuth;

#[test]
fn parse_basic_auth_rfc() {
    let mut map = http::HeaderMap::new();
    map.insert(
        http::header::AUTHORIZATION,
        http::HeaderValue::from_static("Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="),
    );

    let auth = BasicAuth::extract(&map).unwrap();
    assert_eq!(auth.username(), "Aladdin");
    assert_eq!(auth.password(), "open sesame");
}

#[test]
fn empty_creds() {
    let creds = ":";
    let encoded = base64_simd::STANDARD.encode_to_string(creds);
    let val = format!("Basic {encoded}");

    let mut map = http::HeaderMap::new();
    map.insert(
        http::header::AUTHORIZATION,
        http::HeaderValue::from_str(val.as_str()).unwrap(),
    );

    let auth = BasicAuth::extract(&map).unwrap();
    assert_eq!(auth.username(), "");
    assert_eq!(auth.password(), "");
}

#[test]
fn empty_username() {
    let creds = ":UwU";
    let encoded = base64_simd::STANDARD.encode_to_string(creds);
    let val = format!("Basic {encoded}");

    let mut map = http::HeaderMap::new();
    map.insert(
        http::header::AUTHORIZATION,
        http::HeaderValue::from_str(val.as_str()).unwrap(),
    );

    let auth = BasicAuth::extract(&map).unwrap();
    assert_eq!(auth.username(), "");
    assert_eq!(auth.password(), "UwU");
}

#[test]
fn empty_password() {
    let creds = "OwO:";
    let encoded = base64_simd::STANDARD.encode_to_string(creds);
    let val = format!("Basic {encoded}");

    let mut map = http::HeaderMap::new();
    map.insert(
        http::header::AUTHORIZATION,
        http::HeaderValue::from_str(val.as_str()).unwrap(),
    );

    let auth = BasicAuth::extract(&map).unwrap();
    assert_eq!(auth.username(), "OwO");
    assert_eq!(auth.password(), "");
}
