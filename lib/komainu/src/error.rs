use thiserror::Error;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Malformed body")]
    Body(#[source] BoxError),

    #[error("Missing parameter")]
    MissingParam,

    #[error("Malformed query")]
    Query(#[source] BoxError),

    #[error("Request is unauthorized")]
    Unauthorized,
}

impl Error {
    #[track_caller]
    pub(crate) fn body(err: impl Into<BoxError>) -> Self {
        Self::Body(err.into())
    }

    #[track_caller]
    pub(crate) fn query(err: impl Into<BoxError>) -> Self {
        Self::Query(err.into())
    }
}
