use http::StatusCode;
use std::borrow::Cow;
use thiserror::Error;

macro_rules! http_error {
    ($($variant_name:ident => $status_code:path),*$(,)?) => {
        #[derive(Debug, Error)]
        pub enum HttpError {
            $(
                #[doc = stringify!($variant_name)]
                #[error("{}", self.as_str())]
                $variant_name,
            )*
        }

        impl HttpError {
            #[inline]
            pub fn as_str(&self) -> Cow<'static, str> {
                let status_code = self.status_code();

                status_code
                    .canonical_reason()
                    .map_or_else(
                        || Cow::Owned(status_code.as_str().to_string()),
                        Cow::Borrowed,
                    )
            }

            #[inline]
            #[must_use]
            pub fn status_code(&self) -> ::http::StatusCode {
                match self {
                    $(
                        Self::$variant_name => $status_code,
                    )*
                }
            }
        }
    }
}

http_error! {
    BadRequest => StatusCode::NOT_FOUND,
    InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
    NotFound => StatusCode::NOT_FOUND,
    Unauthorised => StatusCode::UNAUTHORIZED,
    UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
}
