use komainu::extract::BasicAuth;
use rstest::rstest;

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

#[rstest]
#[case("", "")]
#[case("OwO", "")]
#[case("", "UwU")]
fn emptiness(#[case] username: &str, #[case] password: &str) {
    let creds = format!("{username}:{password}");
    let encoded = base64_simd::STANDARD.encode_to_string(creds);
    let val = format!("Basic {encoded}");

    let mut map = http::HeaderMap::new();
    map.insert(
        http::header::AUTHORIZATION,
        http::HeaderValue::from_str(val.as_str()).unwrap(),
    );

    let auth = BasicAuth::extract(&map).unwrap();
    assert_eq!(auth.username(), username);
    assert_eq!(auth.password(), password);
}

#[rstest]
#[case("pwease?? 🥺", "enter?? 👉👈")]
#[case("hällö 💀", "tschüß 😔")]
#[case("안녕하세요", "안녕")]
#[case("مرحبًا", "مع السلامة")]
#[case("🏳️‍⚧️", "🏳‍🌈")]
fn unicode_works(#[case] username: &str, #[case] password: &str) {
    let creds = format!("{username}:{password}");
    let encoded = base64_simd::STANDARD.encode_to_string(creds);
    let val = format!("Basic {encoded}");

    let mut map = http::HeaderMap::new();
    map.insert(
        http::header::AUTHORIZATION,
        http::HeaderValue::from_str(val.as_str()).unwrap(),
    );

    let auth = BasicAuth::extract(&map).unwrap();
    assert_eq!(auth.username(), username);
    assert_eq!(auth.password(), password);
}
