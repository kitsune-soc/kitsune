use bytes::Bytes;
use http::Request;
use http_body_util::{BodyExt, Full};
use std::convert::Infallible;
use tower::{service_fn, ServiceExt};
use tower_http_digest::{VerifyDigestBody, VerifyDigestLayer};
use tower_layer::Layer;
use tower_service::Service;

const TEXT: &str = r"Una sombra abajo de mi cama
Cómo se llama éste fantasma?
Es mi espejo, es mi espejo
Y me persigue hasta que hablemos";

const EXPECTED_SHA256_HASH: &str = "vDI/NDnFX991qKsNsKB5Ne4bam8J5eLLYqo0jU8ku+I=";
const EXPECTED_SHA512_HASH: &str =
    "zTNHlXez9GjaWU8Z/7OM6ntFjCbxcOfuc7NRp8F4m3fVrmG5K/7QST2lQiif8EGEopqih9eFlbo0dumbsBYP4g==";

#[test]
fn digest_invalid_base64() {
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

    let mut service = VerifyDigestLayer::default().layer(service_fn(
        |request: Request<VerifyDigestBody<Full<Bytes>>>| async move {
            assert!(request.collect().await.is_err());
            Ok::<_, Infallible>(())
        },
    ));

    futures::executor::block_on(async move {
        service.ready().await.unwrap().call(request).await.unwrap();
    });
}

#[test]
fn digest_invalid_no_base64() {
    let request = Request::builder()
        .header("digest", "sha-256=THIS-IS-BAD")
        .body(Full::from(TEXT))
        .unwrap();

    let mut service = VerifyDigestLayer::default().layer(service_fn(
        |_request: Request<VerifyDigestBody<Full<Bytes>>>| async move { Ok::<_, Infallible>(()) },
    ));

    futures::executor::block_on(async move {
        assert!(service.ready().await.unwrap().call(request).await.is_err());
    });
}

#[test]
fn digest_sha256() {
    let request = Request::builder()
        .header("digest", format!("sha-256={EXPECTED_SHA256_HASH}"))
        .body(Full::from(TEXT))
        .unwrap();

    let mut service = VerifyDigestLayer::default().layer(service_fn(
        |request: Request<VerifyDigestBody<Full<Bytes>>>| async move {
            let body = request.collect().await.unwrap().to_bytes();
            assert_eq!(body, TEXT);
            Ok::<_, Infallible>(())
        },
    ));

    futures::executor::block_on(async move {
        service.ready().await.unwrap().call(request).await.unwrap();
    });
}

#[test]
fn digest_sha512() {
    let request = Request::builder()
        .header("digest", format!("sha-512={EXPECTED_SHA512_HASH}"))
        .body(Full::from(TEXT))
        .unwrap();

    let mut service = VerifyDigestLayer::default().layer(service_fn(
        |request: Request<VerifyDigestBody<Full<Bytes>>>| async move {
            let body = request.collect().await.unwrap().to_bytes();
            assert_eq!(body, TEXT);

            Ok::<_, Infallible>(())
        },
    ));

    futures::executor::block_on(async move {
        service.ready().await.unwrap().call(request).await.unwrap();
    });
}
