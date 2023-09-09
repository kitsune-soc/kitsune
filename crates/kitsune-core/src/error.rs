use thiserror::Error;
use tokio::sync::oneshot;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum FederationFilterError {
    #[error(transparent)]
    Glob(#[from] globset::Error),

    #[error("Host missing from URL")]
    HostMissing,

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Cache(#[from] kitsune_cache::Error),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    FederationFilter(#[from] FederationFilterError),

    #[error(transparent)]
    HttpClient(#[from] kitsune_http_client::Error),

    #[error(transparent)]
    OAuth2(#[from] OAuth2Error),

    #[error(transparent)]
    TokioOneshot(#[from] oneshot::error::RecvError),
}

#[derive(Debug, Error)]
pub enum OAuth2Error {
    #[error("Missing grant type")]
    MissingGrantType,

    #[error(transparent)]
    OxideAuth(#[from] oxide_auth::endpoint::OAuthError),

    #[error("Unknown grant type")]
    UnknownGrantType,

    #[error(transparent)]
    Web(#[from] oxide_auth_axum::WebError),
}
