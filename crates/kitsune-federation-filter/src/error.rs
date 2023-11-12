use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Glob(#[from] globset::Error),

    #[error("Host missing from URL")]
    HostMissing,

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}
