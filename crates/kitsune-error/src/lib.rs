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
        return Err($crate::kitsune_error!($(type = $type,)? $msg));
    };
}

#[macro_export]
macro_rules! kitsune_error {
    (type = $type:expr, $msg:expr) => {
        $crate::Error::msg($msg).with_error_type($type)
    };
    ($msg:expr) => {
        $crate::kitsune_error!(type = $crate::ErrorType::Other(None), $msg)
    };
}

#[derive(Clone, Debug)]
pub enum ErrorType {
    BadRequest(Option<String>),
    Forbidden(Option<String>),
    NotFound,
    Unauthorized,
    UnsupportedMediaType,
    Other(Option<String>),
}

#[derive(Debug)]
pub struct Error {
    ty: ErrorType,
    inner: eyre::Report,
}

impl Error {
    #[inline]
    pub fn new<E>(ty: ErrorType, err: E) -> Self
    where
        E: Into<eyre::Report>,
    {
        Self {
            ty,
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
    pub fn error_type(&self) -> &ErrorType {
        &self.ty
    }

    pub fn error(&self) -> &eyre::Report {
        &self.inner
    }

    #[must_use]
    pub fn with_error_type(self, ty: ErrorType) -> Self {
        Self { ty, ..self }
    }
}

impl<T> From<T> for Error
where
    T: Into<eyre::Report>,
{
    fn from(value: T) -> Self {
        Self {
            ty: ErrorType::Other(None),
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
