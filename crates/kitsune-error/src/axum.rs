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

macro_rules! dispatch_response {
    ($value:expr, $body:expr; {
        $($error_ty:pat => $status_code:expr),* $(,)?
    }) => {{
        match $value {
            $(
                $error_ty => to_response($status_code, $body),
            )*
        }
    }};
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        debug!(error = ?self.inner);

        if let Some(garde_report) = self.error().downcast_ref::<garde::Report>() {
            let body = match sonic_rs::to_string(&garde_report) {
                Ok(body) => body,
                Err(error) => return Error::from(error).into_response(),
            };

            return to_response(StatusCode::BAD_REQUEST, Some(body));
        }

        dispatch_response!(self.context().ty, self.into_context().body.into_inner(); {
            ErrorType::BadRequest => StatusCode::BAD_REQUEST,
            ErrorType::Forbidden => StatusCode::FORBIDDEN,
            ErrorType::NotFound => StatusCode::NOT_FOUND,
            ErrorType::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorType::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ErrorType::Other => StatusCode::INTERNAL_SERVER_ERROR,
        })
    }
}
