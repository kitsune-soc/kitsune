use self::fixtures::Fixture;
use bytes::Bytes;
use http_body_util::Empty;
use komainu::scope::Scope;
use serde::Serialize;
use std::str::FromStr;

mod fixtures;

#[allow(clippy::unreadable_literal)]
const RNG_SEED: u64 = 0xBADD1E;

#[derive(Serialize)]
struct SerdeResponse {
    body: Option<String>,
    #[serde(with = "http_serde::header_map")]
    headers: http::HeaderMap,
    #[serde(with = "http_serde::status_code")]
    status: http::StatusCode,
}

impl From<http::Response<()>> for SerdeResponse {
    #[inline]
    fn from(value: http::Response<()>) -> Self {
        let (parts, _body) = value.into_parts();

        Self {
            body: None,
            headers: parts.headers,
            status: parts.status,
        }
    }
}

impl From<http::Response<Bytes>> for SerdeResponse {
    #[inline]
    fn from(value: http::Response<Bytes>) -> Self {
        let (parts, body) = value.into_parts();
        let body = String::from_utf8(body.to_vec()).unwrap();

        let mut response: Self = http::Response::from_parts(parts, ()).into();
        response.body = Some(body);
        response
    }
}

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
