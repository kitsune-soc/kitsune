#[macro_use]
extern crate tracing;

pub use self::ext::ResultExt;

mod axum;
mod ext;

#[derive(Clone)]
pub enum ErrorType {
    BadRequest(Option<String>),
    NotFound,
    Unauthorized,
    Other,
}

pub struct Error {
    ty: ErrorType,
    inner: eyre::Report,
}

impl Error {
    pub fn new<E>(ty: ErrorType, err: E) -> Self
    where
        E: Into<eyre::Report>,
    {
        Self {
            ty,
            inner: err.into(),
        }
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
            ty: ErrorType::Other,
            inner: value.into(),
        }
    }
}
