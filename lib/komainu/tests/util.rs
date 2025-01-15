use bytes::Bytes;
use serde::Serialize;

#[derive(Serialize)]
pub struct SerdeResponse {
    pub body: Option<String>,
    #[serde(with = "http_serde::header_map")]
    pub headers: http::HeaderMap,
    #[serde(with = "http_serde::status_code")]
    pub status: http::StatusCode,
}

impl From<http::Response<()>> for SerdeResponse {
    #[inline]
    fn from(value: http::Response<()>) -> Self {
        let (parts, _body) = value.into_parts();

        Self {
            body: None,
            headers: parts.headers,
            status: parts.status,
        }
    }
}

impl From<http::Response<Bytes>> for SerdeResponse {
    #[inline]
    fn from(value: http::Response<Bytes>) -> Self {
        let (parts, body) = value.into_parts();
        let body = String::from_utf8(body.to_vec()).unwrap();

        let mut response: Self = http::Response::from_parts(parts, ()).into();
        response.body = Some(body);
        response
    }
}
