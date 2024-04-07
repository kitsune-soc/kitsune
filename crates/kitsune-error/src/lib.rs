#[macro_use]
extern crate tracing;

use std::fmt::{self, Debug, Display};

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

#[derive(Clone, Debug)]
pub enum ErrorType {
    BadRequest,
    Forbidden,
    NotFound,
    Unauthorized,
    UnsupportedMediaType,
    Other,
}

impl ErrorType {
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn with_body<B>(self, body: B) -> ErrorContext
    where
        B: ToString,
    {
        ErrorContext {
            ty: self,
            body: Some(body.to_string()),
        }
    }
}

impl From<ErrorType> for ErrorContext {
    fn from(value: ErrorType) -> Self {
        Self {
            ty: value,
            body: None,
        }
    }
}

#[derive(Debug)]
pub struct ErrorContext {
    ty: ErrorType,
    body: Option<String>,
}

#[derive(Debug)]
pub struct Error {
    ctx: ErrorContext,
    inner: eyre::Report,
}

impl Error {
    #[inline]
    pub fn new<E>(ctx: ErrorContext, err: E) -> Self
    where
        E: Into<eyre::Report>,
    {
        Self {
            ctx,
            inner: err.into(),
        }
    }

    #[inline]
    pub fn msg<M>(msg: M) -> Self
    where
        M: Debug + Display + Send + Sync + 'static,
    {
        eyre::Report::msg(msg).into()
    }

    #[must_use]
    pub fn context(&self) -> &ErrorContext {
        &self.ctx
    }

    pub fn error(&self) -> &eyre::Report {
        &self.inner
    }

    pub fn into_error(self) -> eyre::Report {
        self.inner
    }

    #[must_use]
    pub fn with_context(self, ctx: ErrorContext) -> Self {
        Self { ctx, ..self }
    }
}

impl<T> From<T> for Error
where
    T: Into<eyre::Report>,
{
    fn from(value: T) -> Self {
        Self {
            ctx: ErrorType::Other.into(),
            inner: value.into(),
        }
    }
}

impl From<Error> for BoxError {
    fn from(value: Error) -> Self {
        BoxError::from(value.inner)
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <eyre::Report as fmt::Display>::fmt(&self.inner, f)
    }
}
