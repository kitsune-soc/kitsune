use axum_core::body::Body;
use http::Request;
use std::convert::Infallible;
use tower::{service_fn, ServiceExt};
use tower_http_digest::{Algorithm, DigestLayer};
use tower_layer::Layer;
use tower_service::Service;

#[test]
fn unlimited_data() {
    let zero_stream = futures::stream::repeat(Ok::<_, Infallible>(vec![0_u8; 512]));
    let request = Request::new(Body::from_stream(zero_stream));

    let mut service =
        DigestLayer::new(Algorithm::Sha256).layer(service_fn(|_request: Request<Body>| async {
            Ok::<_, Infallible>(())
        }));

    futures::executor::block_on(async move {
        service
            .ready()
            .await
            .unwrap()
            .call(request)
            .await
            .expect_err("This should error out rather quickly");
    });
}
