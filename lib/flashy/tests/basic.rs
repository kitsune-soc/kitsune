use flashy::{FlashHandle, FlashLayer, IncomingFlashes, Key, Level};
use http::header::{COOKIE, SET_COOKIE};
use std::convert::Infallible;
use tower::{Layer, ServiceExt};

const KEY: &[u8] = b"it's a femininomenon! hit it like rom-pom-pom-pom!";

#[futures_test::test]
async fn roundtrip() {
    let flash_layer = FlashLayer::new(Key::derive_from(KEY));

    let service = flash_layer.layer(tower::service_fn(|req: http::Request<()>| async move {
        let extracted = req.extensions().get::<IncomingFlashes>().unwrap();
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
        let extracted = req.extensions().get::<IncomingFlashes>().unwrap();
        assert!(!extracted.is_empty());

        let mut iter = extracted.iter();
        assert_eq!(iter.next(), Some((Level::Success, "YAY")));

        Ok::<_, Infallible>(http::Response::new(()))
    }));

    service.oneshot(request).await.unwrap();
}

#[futures_test::test]
async fn reject_invalid_signature() {
    let correct_cookie_header = "FLASHY-FLASH_MESSAGES=b59fa319105e34641efc1dfd2be232c15e8d8be85c5bfb8c96c7aa17e581eab4.%5B%5B%22Success%22%2C%22YAY%22%5D%5D; SameSite=Strict; Secure";

    // we'll make it evil >:3
    // instead of being "success" and "YAY", we'll make it "Error" and "NAY" :33
    let wrong_cookie_header = correct_cookie_header
        .replace("Success", "Error")
        .replace("YAY", "NAY");

    let flash_layer = FlashLayer::new(Key::derive_from(KEY));

    let request = http::Request::builder()
        .header(COOKIE, wrong_cookie_header)
        .body(())
        .unwrap();

    let service = flash_layer.layer(tower::service_fn(|req: http::Request<()>| async move {
        // this should be empty because we ignore tampered with messages
        let extracted = req.extensions().get::<IncomingFlashes>().unwrap();
        assert!(extracted.is_empty());

        Ok::<_, Infallible>(http::Response::new(()))
    }));

    service.oneshot(request).await.unwrap();

    let request = http::Request::builder()
        .header(COOKIE, correct_cookie_header)
        .body(())
        .unwrap();

    let service = flash_layer.layer(tower::service_fn(|req: http::Request<()>| async move {
        let extracted = req.extensions().get::<IncomingFlashes>().unwrap();
        assert_eq!(extracted.len(), 1);

        Ok::<_, Infallible>(http::Response::new(()))
    }));

    service.oneshot(request).await.unwrap();
}
