use axum::{
    middleware::Next,
    response::{IntoResponse, Response},
    RequestExt,
};
use bytes::Buf;
use headers::{ContentType, HeaderMapExt};
use http::{Request, StatusCode};
use hyper::Body;
use simd_json::OwnedValue;

pub async fn json_to_urlencoded(req: Request<Body>, next: Next<Body>) -> Response {
    if req.headers().typed_get::<ContentType>() != Some(ContentType::json()) {
        return next.run(req).await;
    }

    let Ok(req) = req.with_limited_body() else {
        panic!("[Bug] Body is not limited. Please fix IMMEDIATELY! (annoy the devs)");
    };
    let (parts, body) = req.into_parts();

    let json_value = match hyper::body::to_bytes(body)
        .await
        .map(|bytes| simd_json::from_reader::<_, OwnedValue>(bytes.reader()))
    {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => {
            error!(?error, "failed to deserialise json body");
            return (StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
        Err(error) => {
            error!(?error, "failed to buffer body into memory");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let body = match serde_urlencoded::to_string(json_value) {
        Ok(reencoded_body) => Body::from(reencoded_body),
        Err(error) => {
            error!(?error, "failed to reencode json body into urlencoded");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut req = Request::from_parts(parts, body);
    req.headers_mut()
        .typed_insert(ContentType::form_url_encoded());

    next.run(req).await
}
