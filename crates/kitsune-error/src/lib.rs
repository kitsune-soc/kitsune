#[macro_use]
extern crate tracing;

use axum_core::response::{IntoResponse, Response};
use std::fmt::{self, Debug, Display};
use sync_wrapper::SyncWrapper;

pub use self::ext::ResultExt;

mod axum;
mod ext;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[macro_export]
macro_rules! bail {
    ($(type = $type:expr,)? $msg:expr) => {
        return Err($crate::kitsune_error!($(type = $type,)? $msg).into());
    };
}

#[macro_export]
macro_rules! kitsune_error {
    (type = $type:expr, $msg:expr) => {
        $crate::Error::msg($msg).with_context({ $type }.into())
    };
    ($msg:expr) => {
        $crate::kitsune_error!(type = $crate::ErrorType::Other, $msg)
    };
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorType {
    BadRequest,
    Forbidden,
    NotFound,
    Unauthorized,
    UnsupportedMediaType,
    Other,
}

impl ErrorType {
    #[inline]
    #[must_use]
    pub fn with_body<B>(self, body: B) -> ErrorContext
    where
        B: IntoResponse,
    {
        ErrorContext {
            ty: self,
            body: Some(body.into_response()).into(),
        }
    }
}

impl From<ErrorType> for ErrorContext {
    #[inline]
    fn from(value: ErrorType) -> Self {
        Self {
            ty: value,
            body: SyncWrapper::new(None),
        }
    }
}

#[derive(Debug)]
pub struct ErrorContext {
    ty: ErrorType,
    body: SyncWrapper<Option<Response>>,
}

#[derive(Debug)]
struct ErrorInner {
    ctx: ErrorContext,
    inner: eyre::Report,
}

pub struct Error {
    inner: Box<ErrorInner>,
}

impl Error {
    #[inline]
    #[track_caller]
    pub fn new<E>(ctx: ErrorContext, err: E) -> Self
    where
        E: Into<eyre::Report>,
    {
        Self {
            inner: Box::new(ErrorInner {
                ctx,
                inner: err.into(),
            }),
        }
    }

    #[inline]
    pub fn msg<M>(msg: M) -> Self
    where
        M: Debug + Display + Send + Sync + 'static,
    {
        eyre::Report::msg(msg).into()
    }

    #[inline]
    #[must_use]
    pub fn context(&self) -> &ErrorContext {
        &self.inner.ctx
    }

    #[inline]
    #[must_use]
    pub fn into_context(self) -> ErrorContext {
        self.inner.ctx
    }

    #[inline]
    pub fn error(&self) -> &eyre::Report {
        &self.inner.inner
    }

    #[inline]
    pub fn into_error(self) -> eyre::Report {
        self.inner.inner
    }

    #[inline]
    #[must_use]
    pub fn with_context(mut self, ctx: ErrorContext) -> Self {
        self.inner.ctx = ctx;
        self
    }
}

impl<T> From<T> for Error
where
    T: Into<eyre::Report>,
{
    #[inline]
    #[track_caller]
    fn from(value: T) -> Self {
        Self::new(ErrorType::Other.into(), value)
    }
}

impl From<Error> for BoxError {
    #[inline]
    fn from(value: Error) -> Self {
        BoxError::from(value.into_error())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <eyre::Report as fmt::Display>::fmt(self.error(), f)
    }
}
