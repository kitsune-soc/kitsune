use cookie::Key;
use flashy::{FlashHandle, FlashLayer, Level, ReadFlashes};
use http::header::{COOKIE, SET_COOKIE};
use std::convert::Infallible;
use tower::{Layer, ServiceExt};

#[futures_test::test]
async fn roundtrip() {
    let flash_layer = FlashLayer::new(Key::generate());

    let service = flash_layer.layer(tower::service_fn(|req: http::Request<()>| async move {
        let extracted = req.extensions().get::<ReadFlashes>().unwrap();
        assert!(extracted.is_empty());

        let handle = req.extensions().get::<FlashHandle>().unwrap();
        handle.push(Level::Success, "YAY");

        Ok::<_, Infallible>(http::Response::new(()))
    }));
    let response = service.oneshot(http::Request::new(())).await.unwrap();

    // put it in again and see if we can read it

    let mut request = http::Request::builder();
    for value in response.headers().get_all(SET_COOKIE) {
        request = request.header(COOKIE, value);
    }
    let request = request.body(()).unwrap();

    let service = flash_layer.layer(tower::service_fn(|req: http::Request<()>| async move {
        let extracted = req.extensions().get::<ReadFlashes>().unwrap();
        assert!(!extracted.is_empty());

        let mut iter = extracted.iter();
        assert_eq!(iter.next(), Some((Level::Success, "YAY")));

        Ok::<_, Infallible>(http::Response::new(()))
    }));

    service.oneshot(request).await.unwrap();
}
