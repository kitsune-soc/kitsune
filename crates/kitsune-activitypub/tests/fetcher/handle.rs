use http::{Request, Response, StatusCode};
use http_body_util::{combinators::BoxBody, Full};
use hyper::body::Bytes;
use kitsune_test::build_ap_response;
use std::convert::Infallible;
use tower::BoxError;

pub async fn handle(
    req: Request<BoxBody<Bytes, BoxError>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match req.uri().path_and_query().unwrap().as_str() {
        "/users/0x0" => {
            let body = include_str!("../../../../test-fixtures/0x0_actor.json");
            Ok::<_, Infallible>(build_ap_response(body))
        }
        "/@0x0/109501674056556919" => {
            let body =
                include_str!("../../../../test-fixtures/corteximplant.com_109501674056556919.json");
            Ok::<_, Infallible>(build_ap_response(body))
        }
        "/users/0x0/statuses/109501659207519785" => {
            let body =
                include_str!("../../../../test-fixtures/corteximplant.com_109501659207519785.json");
            Ok::<_, Infallible>(build_ap_response(body))
        }
        "/emojis/7952" => {
            let body = include_str!("../../../../test-fixtures/corteximplant.com_emoji_7952.json");
            Ok::<_, Infallible>(build_ap_response(body))
        }
        "/emojis/8933" => {
            let body = include_str!("../../../../test-fixtures/corteximplant.com_emoji_8933.json");
            Ok::<_, Infallible>(build_ap_response(body))
        }
        "/.well-known/webfinger?resource=acct:0x0@corteximplant.com" => {
            let body = include_str!("../../../../test-fixtures/0x0_jrd.json");
            Ok::<_, Infallible>(Response::new(Full::new(body.as_bytes().into())))
        }
        path if path.starts_with("/.well-known/webfinger?") => Ok::<_, Infallible>(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::default())
                .unwrap(),
        ),
        path => panic!("HTTP client hit unexpected route: {path}"),
    }
}
