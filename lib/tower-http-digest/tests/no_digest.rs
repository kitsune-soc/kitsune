use axum_core::body::Body;
use http::Request;
use std::convert::Infallible;
use tower::{service_fn, ServiceExt};
use tower_http_digest::{Algorithm, DigestLayer};
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
fn no_digest_sha256() {
    let request = Request::new(Body::from(TEXT));
    let mut service = DigestLayer::new(Algorithm::Sha256).layer(service_fn(
        |request: Request<Body>| async move {
            let (algorithm, hash) = request
                .headers()
                .get("digest")
                .unwrap()
                .to_str()
                .unwrap()
                .split_once('=')
                .unwrap();

            assert_eq!(algorithm, "sha-256");
            assert_eq!(hash, EXPECTED_SHA256_HASH);

            Ok::<_, Infallible>(())
        },
    ));

    futures::executor::block_on(async move {
        service.ready().await.unwrap().call(request).await.unwrap();
    });
}

#[test]
fn no_digest_sha512() {
    let request = Request::new(Body::from(TEXT));
    let mut service = DigestLayer::new(Algorithm::Sha512).layer(service_fn(
        |request: Request<Body>| async move {
            let (algorithm, hash) = request
                .headers()
                .get("digest")
                .unwrap()
                .to_str()
                .unwrap()
                .split_once('=')
                .unwrap();

            assert_eq!(algorithm, "sha-512");
            assert_eq!(hash, EXPECTED_SHA512_HASH);

            Ok::<_, Infallible>(())
        },
    ));

    futures::executor::block_on(async move {
        service.ready().await.unwrap().call(request).await.unwrap();
    });
}
