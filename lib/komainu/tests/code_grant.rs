use self::{fixtures::Fixture, util::SerdeResponse};
use bytes::Bytes;
use http_body_util::Empty;
use komainu::{code_grant::GrantError, scope::Scope};
use std::str::FromStr;

mod fixtures;
mod util;

#[allow(clippy::unreadable_literal)]
const RNG_SEED: u64 = 0xBADD1E;

#[futures_test::test]
async fn success() {
    fastrand::seed(RNG_SEED);
    let fixture = Fixture::generate();

    let uri = http::Uri::builder()
        .scheme("http")
        .authority("komainu.example")
        .path_and_query("/oauth/authorize?response_type=code&client_id=client_1")
        .build()
        .unwrap();

    let req = http::Request::builder()
        .uri(uri)
        .body(Empty::<Bytes>::new())
        .unwrap();
    let req = komainu::Request::read_from(req).await.unwrap();

    let acceptor = {
        let handle = fixture.code_grant.extract_raw(&req).await.unwrap();
        handle
            .accept("user id".into(), &Scope::from_str("read").unwrap())
            .await
            .unwrap()
    };

    let deny = {
        let handle = fixture.code_grant.extract_raw(&req).await.unwrap();
        handle.deny()
    };

    insta::assert_json_snapshot!(SerdeResponse::from(acceptor.into_response()));
    insta::assert_json_snapshot!(SerdeResponse::from(deny));
}

#[futures_test::test]
async fn redirect_uri_mismatch() {
    fastrand::seed(RNG_SEED);
    let fixture = Fixture::generate();

    let uri = http::Uri::builder()
        .scheme("http")
        .authority("komainu.example")
        .path_and_query("/oauth/authorize?response_type=code&client_id=client_1&redirect_uri=http%3A%2F%2Fbad-redirect-uri.example%2Fhehe")
        .build()
        .unwrap();

    let req = http::Request::builder()
        .uri(uri)
        .body(Empty::<Bytes>::new())
        .unwrap();
    let req = komainu::Request::read_from(req).await.unwrap();

    let err = match fixture.code_grant.extract_raw(&req).await {
        Ok(..) => panic!(),
        Err(err) => err,
    };

    assert!(matches!(err, GrantError::AccessDenied));
}
