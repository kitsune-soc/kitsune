use cursiv::{CsrfHandle, CsrfLayer, Message};
use futures::{executor, future};
use http::{header, Request, Response};
use std::convert::Infallible;
use tower::{service_fn, Layer, Service, ServiceExt};

fn common() -> impl Service<Request<()>, Response = Response<()>, Error = Infallible> {
    let key = blake3::derive_key("wawa", b"bartmoss");
    let service = service_fn(|req: Request<()>| {
        let handle = req.extensions().get::<CsrfHandle>().unwrap();
        if let Some(msg) = req.extensions().get::<Message>() {
            assert!(handle.verify(msg), "BAD VERIFICATION");
        }

        let msg = handle.sign("SESSION_ID");

        let mut resp = Response::new(());
        resp.extensions_mut().insert(msg);

        future::ok::<_, Infallible>(resp)
    });

    CsrfLayer::new(key).layer(service)
}

#[test]
#[should_panic = "BAD VERIFICATION"]
fn panic_wrong_signature() {
    let mut service = common();

    let response = executor::block_on(async {
        service
            .ready()
            .await
            .unwrap()
            .call(Request::default())
            .await
            .unwrap()
    });

    let req = Request::builder()
        .header(
            header::COOKIE,
            response.headers().get(header::SET_COOKIE).unwrap().clone(),
        )
        .extension(Message::from_static("I AM NOT THE SESSION ID"))
        .body(())
        .unwrap();

    executor::block_on(async { service.ready().await.unwrap().call(req).await.unwrap() });
}

#[test]
fn sign_verify_simple() {
    let mut service = common();
    let response = executor::block_on(async {
        service
            .ready()
            .await
            .unwrap()
            .call(Request::default())
            .await
            .unwrap()
    });

    let req = Request::builder()
        .header(
            header::COOKIE,
            response.headers().get(header::SET_COOKIE).unwrap().clone(),
        )
        .extension(response.extensions().get::<Message>().unwrap().clone())
        .body(())
        .unwrap();

    executor::block_on(async { service.ready().await.unwrap().call(req).await.unwrap() });
}
