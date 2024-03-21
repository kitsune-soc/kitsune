use bytes::Bytes;
use http::{Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use std::convert::Infallible;
use tower::{service_fn, ServiceExt};
use tower_http_digest::{VerifyDigestBody, VerifyDigestLayer};
use tower_layer::Layer;

const TEXT: &str = r"Una sombra abajo de mi cama
Cómo se llama éste fantasma?
Es mi espejo, es mi espejo
Y me persigue hasta que hablemos";

const EXPECTED_SHA256_HASH: &str = "vDI/NDnFX991qKsNsKB5Ne4bam8J5eLLYqo0jU8ku+I=";
const EXPECTED_SHA512_HASH: &str =
    "zTNHlXez9GjaWU8Z/7OM6ntFjCbxcOfuc7NRp8F4m3fVrmG5K/7QST2lQiif8EGEopqih9eFlbo0dumbsBYP4g==";

#[futures_test::test]
async fn missing_header() {
    let service = VerifyDigestLayer::default().layer(service_fn(
        |_request: Request<VerifyDigestBody<Full<Bytes>>>| {
            #[allow(unreachable_code)]
            async move {
                unreachable!() as Result<Response<Full<Bytes>>, Infallible>
            }
        },
    ));

    let response = service
        .oneshot(Request::new(Full::from(TEXT)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[futures_test::test]
async fn can_handle_one_invalid() {
    let request = Request::builder()
        .header(
            "digest",
            format!("made-up-hash=woowee,sha-512={EXPECTED_SHA512_HASH}"),
        )
        .body(Full::from(TEXT))
        .unwrap();

    let service = VerifyDigestLayer::default().layer(service_fn(
        |request: Request<VerifyDigestBody<Full<Bytes>>>| async move {
            let body = request.collect().await.unwrap().to_bytes();
            assert_eq!(body, TEXT);
            Ok::<_, Infallible>(Response::<Full<Bytes>>::default())
        },
    ));

    let response = service.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[futures_test::test]
async fn can_handle_multiple() {
    let request = Request::builder()
        .header(
            "digest",
            format!("sha-256={EXPECTED_SHA256_HASH},sha-512={EXPECTED_SHA512_HASH}"),
        )
        .body(Full::from(TEXT))
        .unwrap();

    let service = VerifyDigestLayer::default().layer(service_fn(
        |request: Request<VerifyDigestBody<Full<Bytes>>>| async move {
            let body = request.collect().await.unwrap().to_bytes();
            assert_eq!(body, TEXT);
            Ok::<_, Infallible>(Response::<Full<Bytes>>::default())
        },
    ));

    let response = service.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[futures_test::test]
async fn digest_invalid_base64() {
    let request = Request::builder()
        .header(
            "digest",
            format!(
                "sha-256={}",
                base64_simd::STANDARD.encode_to_string("WHATEVER")
            ),
        )
        .body(Full::from(TEXT))
        .unwrap();

    let service = VerifyDigestLayer::default().layer(service_fn(
        |request: Request<VerifyDigestBody<Full<Bytes>>>| async move {
            assert!(request.collect().await.is_err());
            Ok::<_, Infallible>(Response::<Full<Bytes>>::default())
        },
    ));

    // The response code is ignored here since the actual validation is done in the HTTP body
    // The above assert ensures that our code actually errors out on mismatch
    service.oneshot(request).await.unwrap();
}

#[futures_test::test]
async fn digest_invalid_no_base64() {
    let request = Request::builder()
        .header("digest", "sha-256=THIS-IS-BAD")
        .body(Full::from(TEXT))
        .unwrap();

    let service = VerifyDigestLayer::default().layer(service_fn(
        |_request: Request<VerifyDigestBody<Full<Bytes>>>| async move {
            Ok::<_, Infallible>(Response::<Full<Bytes>>::default())
        },
    ));

    let response = service.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[futures_test::test]
async fn digest_sha256() {
    let request = Request::builder()
        .header("digest", format!("sha-256={EXPECTED_SHA256_HASH}"))
        .body(Full::from(TEXT))
        .unwrap();

    let service = VerifyDigestLayer::default().layer(service_fn(
        |request: Request<VerifyDigestBody<Full<Bytes>>>| async move {
            let body = request.collect().await.unwrap().to_bytes();
            assert_eq!(body, TEXT);
            Ok::<_, Infallible>(Response::<Full<Bytes>>::default())
        },
    ));

    let response = service.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[futures_test::test]
async fn digest_sha512() {
    let request = Request::builder()
        .header("digest", format!("sha-512={EXPECTED_SHA512_HASH}"))
        .body(Full::from(TEXT))
        .unwrap();

    let service = VerifyDigestLayer::default().layer(service_fn(
        |request: Request<VerifyDigestBody<Full<Bytes>>>| async move {
            let body = request.collect().await.unwrap().to_bytes();
            assert_eq!(body, TEXT);

            Ok::<_, Infallible>(Response::<Full<Bytes>>::default())
        },
    ));

    let response = service.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
