use self::{
    fixtures::{Fixture, RNG_SEED},
    util::SerdeResponse,
};
use bytes::Bytes;
use http_body_util::Empty;
use komainu::{code_grant::GrantError, flow::pkce, scope::Scope};
use std::str::FromStr;

mod fixtures;
mod util;

#[futures_test::test]
async fn success_basic() {
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
async fn success_with_state() {
    fastrand::seed(RNG_SEED);
    let fixture = Fixture::generate();

    let uri = http::Uri::builder()
        .scheme("http")
        .authority("komainu.example")
        .path_and_query("/oauth/authorize?response_type=code&client_id=client_1&state=cat%20tax")
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
        Ok(..) => unreachable!(),
        Err(err) => err,
    };

    assert!(matches!(err, GrantError::AccessDenied));
}

#[futures_test::test]
async fn invalid_response_type() {
    fastrand::seed(RNG_SEED);
    let fixture = Fixture::generate();

    let uri = http::Uri::builder()
        .scheme("http")
        .authority("komainu.example")
        .path_and_query("/oauth/authorize?response_type=meoww&client_id=client_1")
        .build()
        .unwrap();

    let req = http::Request::builder()
        .uri(uri)
        .body(Empty::<Bytes>::new())
        .unwrap();
    let req = komainu::Request::read_from(req).await.unwrap();

    let err = match fixture.code_grant.extract_raw(&req).await {
        Ok(..) => unreachable!(),
        Err(err) => err,
    };

    assert!(matches!(err, GrantError::AccessDenied));
}

#[futures_test::test]
async fn success_matching_scopes() {
    fastrand::seed(RNG_SEED);
    let fixture = Fixture::generate();

    let uri = http::Uri::builder()
        .scheme("http")
        .authority("komainu.example")
        .path_and_query("/oauth/authorize?response_type=code&client_id=client_1&scope=read%20write")
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

    insta::assert_json_snapshot!(SerdeResponse::from(acceptor.into_response()));
}

#[futures_test::test]
async fn too_many_scopes() {
    fastrand::seed(RNG_SEED);
    let fixture = Fixture::generate();

    let uri = http::Uri::builder()
        .scheme("http")
        .authority("komainu.example")
        .path_and_query(
            "/oauth/authorize?response_type=code&client_id=client_1&scope=read%20write%20follow",
        )
        .build()
        .unwrap();

    let req = http::Request::builder()
        .uri(uri)
        .body(Empty::<Bytes>::new())
        .unwrap();
    let req = komainu::Request::read_from(req).await.unwrap();

    let err = match fixture.code_grant.extract_raw(&req).await {
        Ok(..) => unreachable!(),
        Err(err) => err,
    };

    assert!(matches!(err, GrantError::AccessDenied));
}

#[futures_test::test]
async fn success_with_pkce_method() {
    fastrand::seed(RNG_SEED);
    let fixture = Fixture::generate();

    let uri = http::Uri::builder()
        .scheme("http")
        .authority("komainu.example")
        .path_and_query("/oauth/authorize?response_type=code&client_id=client_1&code_challenge=challenge&challenge_code_method=none")
        .build()
        .unwrap();

    let req = http::Request::builder()
        .uri(uri)
        .body(Empty::<Bytes>::new())
        .unwrap();
    let req = komainu::Request::read_from(req).await.unwrap();

    let handle = fixture.code_grant.extract_raw(&req).await.unwrap();
    let pkce = handle.pkce().unwrap();

    assert_eq!(pkce.challenge, "challenge");
    assert_eq!(pkce.method, pkce::Method::None);
}

#[futures_test::test]
async fn success_without_pkce_method() {
    fastrand::seed(RNG_SEED);
    let fixture = Fixture::generate();

    let uri = http::Uri::builder()
        .scheme("http")
        .authority("komainu.example")
        .path_and_query(
            "/oauth/authorize?response_type=code&client_id=client_1&code_challenge=challenge",
        )
        .build()
        .unwrap();

    let req = http::Request::builder()
        .uri(uri)
        .body(Empty::<Bytes>::new())
        .unwrap();
    let req = komainu::Request::read_from(req).await.unwrap();

    let handle = fixture.code_grant.extract_raw(&req).await.unwrap();
    let pkce = handle.pkce().unwrap();

    assert_eq!(pkce.challenge, "challenge");
    assert_eq!(pkce.method, pkce::Method::None);
}
