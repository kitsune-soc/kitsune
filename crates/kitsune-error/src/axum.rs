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

impl From<Error> for Response {
    #[inline]
    fn from(value: Error) -> Self {
        value.into_response()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        debug!(error = ?self.inner);

        if let Some(garde_report) = self.inner.downcast_ref::<garde::Report>() {
            let body = match simd_json::to_string(&garde_report) {
                Ok(body) => body,
                Err(error) => return Error::from(error).into_response(),
            };

            return to_response(StatusCode::BAD_REQUEST, Some(body));
        }

        match self.ty {
            ErrorType::BadRequest(maybe_body) => to_response(StatusCode::BAD_REQUEST, maybe_body),
            ErrorType::Forbidden(maybe_body) => to_response(StatusCode::FORBIDDEN, maybe_body),
            ErrorType::NotFound => StatusCode::NOT_FOUND.into_response(),
            ErrorType::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            ErrorType::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response(),
            ErrorType::Other(maybe_body) => {
                to_response(StatusCode::INTERNAL_SERVER_ERROR, maybe_body)
            }
        }
    }
}
