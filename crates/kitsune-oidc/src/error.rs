use openidconnect::{
    core::CoreErrorResponseType, ClaimsVerificationError, DiscoveryError, RequestTokenError,
    SigningError, StandardErrorResponse,
};
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ClaimsVerification(#[from] ClaimsVerificationError),

    #[error(transparent)]
    CreateRedisPool(#[from] deadpool_redis::CreatePoolError),

    #[error(transparent)]
    Discovery(#[from] DiscoveryError<kitsune_http_client::Error>),

    #[error(transparent)]
    JsonParse(#[from] simd_json::Error),

    #[error("Missing Email address")]
    MissingEmail,

    #[error("Mismatching hash")]
    MismatchingHash,

    #[error("Missing ID token")]
    MissingIdToken,

    #[error("Missing login state")]
    MissingLoginState,

    #[error("Missing username")]
    MissingUsername,

    #[error(transparent)]
    Redis(#[from] redis::RedisError),

    #[error(transparent)]
    RedisPool(#[from] deadpool_redis::PoolError),

    #[error(transparent)]
    RequestToken(
        #[from]
        RequestTokenError<
            kitsune_http_client::Error,
            StandardErrorResponse<CoreErrorResponseType>,
        >,
    ),

    #[error(transparent)]
    Signing(#[from] SigningError),

    #[error("Unknown CSRF token")]
    UnknownCsrfToken,

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}
