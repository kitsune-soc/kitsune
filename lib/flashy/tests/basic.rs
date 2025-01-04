use cookie::Key;
use flashy::{FlashHandle, FlashLayer, IncomingFlashes, Level};
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
    // this is correct:
    // "FLASH_MESSAGES=Fk6s+EEdVwkMmBXJakw2oDlSZaErtG+6aHqAViCDhZ8%3D%5B%5B%22Success%22%2C%22YAY%22%5D%5D; SameSite=Strict; Secure"
    //
    // but we'll make it evil >:3
    // instead of being "success" and "YAY", we'll make it "Error" and "NAY" :33
    let cookie_header = "FLASH_MESSAGES=Fk6s+EEdVwkMmBXJakw2oDlSZaErtG+6aHqAViCDhZ8%3D%5B%5B%22Error%22%2C%22NAY%22%5D%5D; SameSite=Strict; Secure";
    let flash_layer = FlashLayer::new(Key::derive_from(KEY));

    let request = http::Request::builder()
        .header(COOKIE, cookie_header)
        .body(())
        .unwrap();

    let service = flash_layer.layer(tower::service_fn(|req: http::Request<()>| async move {
        // this should be empty because we ignore tampered with messages
        let extracted = req.extensions().get::<IncomingFlashes>().unwrap();
        assert!(extracted.is_empty());

        Ok::<_, Infallible>(http::Response::new(()))
    }));

    service.oneshot(request).await.unwrap();
}
