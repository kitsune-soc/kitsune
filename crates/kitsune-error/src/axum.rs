use crate::{Error, ErrorType};
use axum_core::response::{IntoResponse, Response};
use http::StatusCode;

#[inline]
fn to_response<B>(status_code: StatusCode, maybe_body: Option<B>) -> Response
where
    B: IntoResponse,
{
    maybe_body.map_or_else(
        || status_code.into_response(),
        |body| (status_code, body).into_response(),
    )
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        debug!(error = ?self.inner);

        match self.ty {
            ErrorType::BadRequest(maybe_body) => to_response(StatusCode::BAD_REQUEST, maybe_body),
            ErrorType::Forbidden(maybe_body) => to_response(StatusCode::FORBIDDEN, maybe_body),
            ErrorType::NotFound => StatusCode::NOT_FOUND.into_response(),
            ErrorType::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            ErrorType::Other(maybe_body) => {
                to_response(StatusCode::INTERNAL_SERVER_ERROR, maybe_body)
            }
        }
    }
}
