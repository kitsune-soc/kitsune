use axum::body::Body;
use axum::{
    middleware::Next,
    response::{IntoResponse, Response},
    RequestExt,
};
use headers::{ContentType, HeaderMapExt};
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use sonic_rs::Value;

/// Some clients send their OAuth credentials as JSON payloads. This against the OAuth2 RFC but alas, we want high compatibility with Mastodon clients
///
/// This middleware deserialises the body into its DOM representation if the header "Content-Type" is set to "application/json" and reencodes it into the URL-encoded version
pub async fn json_to_urlencoded(req: Request<Body>, next: Next) -> Response {
    if req.headers().typed_get::<ContentType>() != Some(ContentType::json()) {
        return next.run(req).await;
    }

    let (parts, body) = req.with_limited_body().into_parts();

    let json_value = match body
        .collect()
        .await
        .map(|bytes| sonic_rs::from_slice::<Value>(&bytes.to_bytes()))
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
