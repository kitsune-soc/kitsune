use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Cache(#[from] kitsune_cache::Error),

    #[error(transparent)]
    HttpClient(#[from] kitsune_http_client::Error),
}
