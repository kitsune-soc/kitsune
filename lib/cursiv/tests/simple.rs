use cursiv::{CsrfHandle, CsrfLayer, Message};
use http::{Request, Response, header};
use std::{convert::Infallible, future};
use tower::{Layer, Service, ServiceExt, service_fn};

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

        future::ready(Ok::<_, Infallible>(resp))
    });

    CsrfLayer::new(key).layer(service)
}

#[futures_test::test]
#[should_panic = "BAD VERIFICATION"]
async fn panic_wrong_signature() {
    let mut service = common().ready_oneshot().await.unwrap();

    let response = service.call(Request::default()).await.unwrap();
    let req = Request::builder()
        .header(
            header::COOKIE,
            response.headers().get(header::SET_COOKIE).unwrap().clone(),
        )
        .extension(Message::from_static("I AM NOT THE SESSION ID"))
        .body(())
        .unwrap();

    service.oneshot(req).await.unwrap();
}

#[futures_test::test]
async fn sign_verify_simple() {
    let mut service = common().ready_oneshot().await.unwrap();
    let response = service.call(Request::default()).await.unwrap();

    let req = Request::builder()
        .header(
            header::COOKIE,
            response.headers().get(header::SET_COOKIE).unwrap().clone(),
        )
        .extension(response.extensions().get::<Message>().unwrap().clone())
        .body(())
        .unwrap();

    service.oneshot(req).await.unwrap();
}
