use argon2::password_hash;
use axum::{
    extract::multipart::MultipartError,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use rsa::{
    pkcs1,
    pkcs8::{self, der, spki},
};
use std::str::ParseBoolError;
use thiserror::Error;
use tokio::sync::oneshot;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Bad request")]
    BadRequest,

    #[error("Email already taken")]
    EmailTaken,

    #[error("Internal server error")]
    InternalServerError,

    #[error("Not found")]
    NotFound,

    #[error("Registrations closed")]
    RegistrationsClosed,

    #[error("Unauthorised")]
    Unauthorised,

    #[error("Unsupported media type")]
    UnsupportedMediaType,

    #[error("Username already taken")]
    UsernameTaken,
}

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
pub enum Oauth2Error {
    #[error("Missing grant type")]
    MissingGrantType,

    #[error(transparent)]
    OxideAuth(#[from] oxide_auth::endpoint::OAuthError),

    #[error("Unknown grant type")]
    UnknownGrantType,

    #[error(transparent)]
    Web(#[from] oxide_auth_axum::WebError),
}

#[cfg(feature = "oidc")]
use openidconnect::{
    core::CoreErrorResponseType, ClaimsVerificationError, RequestTokenError, SigningError,
    StandardErrorResponse,
};

#[cfg(feature = "oidc")]
#[derive(Debug, Error)]
pub enum OidcError {
    #[error(transparent)]
    ClaimsVerification(#[from] ClaimsVerificationError),

    #[error(transparent)]
    LoginState(#[from] kitsune_cache::Error),

    #[error("Missing Email address")]
    MissingEmail,

    #[error("Mismatching hash")]
    MismatchingHash,

    #[error("Missing ID token")]
    MissingIdToken,

    #[error("Missing username")]
    MissingUsername,

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
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Api(#[from] ApiError),

    #[error("Broken database record encountered")]
    BrokenRecord,

    #[error(transparent)]
    Cache(#[from] kitsune_cache::Error),

    #[error(transparent)]
    Database(#[from] diesel::result::Error),

    #[error(transparent)]
    DatabasePool(#[from] diesel_async::pooled_connection::deadpool::PoolError),

    #[error(transparent)]
    Der(#[from] der::Error),

    #[error(transparent)]
    Event(kitsune_messaging::BoxError),

    #[error(transparent)]
    FederationFilter(#[from] FederationFilterError),

    #[error(transparent)]
    Http(#[from] http::Error),

    #[error(transparent)]
    HttpClient(#[from] kitsune_http_client::Error),

    #[error(transparent)]
    HttpSignature(#[from] kitsune_http_signatures::Error),

    #[error(transparent)]
    KeyRejected(#[from] kitsune_http_signatures::ring::error::KeyRejected),

    #[error("Malformed ActivityPub object")]
    MalformedApObject,

    #[error(transparent)]
    Mime(#[from] mime::FromStrError),

    #[error(transparent)]
    Multipart(#[from] MultipartError),

    #[error(transparent)]
    OAuth(#[from] Oauth2Error),

    #[error(transparent)]
    ParseBool(#[from] ParseBoolError),

    #[error(transparent)]
    Oneshot(#[from] oneshot::error::RecvError),

    #[error(transparent)]
    Rsa(#[from] rsa::errors::Error),

    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

    #[cfg(feature = "oidc")]
    #[error(transparent)]
    Oidc(#[from] OidcError),

    #[error(transparent)]
    ParseScope(#[from] oxide_auth::primitives::scope::ParseScopeErr),

    #[error(transparent)]
    PasswordHash(#[from] password_hash::Error),

    #[error("Password mismatch")]
    PasswordMismatch,

    #[error(transparent)]
    Pkcs1(#[from] pkcs1::Error),

    #[error(transparent)]
    Pkcs8(#[from] pkcs8::Error),

    #[error(transparent)]
    PostProcessing(post_process::BoxError),

    #[error(transparent)]
    Search(#[from] kitsune_search::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Spki(#[from] spki::Error),

    #[error(transparent)]
    Storage(kitsune_storage::BoxError),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),
}

impl From<Error> for Response {
    fn from(err: Error) -> Response {
        err.into_response()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::Database(diesel::result::Error::NotFound) => {
                StatusCode::NOT_FOUND.into_response()
            }
            err @ Self::Api(ApiError::NotFound) => {
                (StatusCode::NOT_FOUND, err.to_string()).into_response()
            }
            err @ Self::Api(ApiError::BadRequest) => {
                (StatusCode::BAD_REQUEST, err.to_string()).into_response()
            }
            err @ (Self::Api(ApiError::Unauthorised) | Self::PasswordMismatch) => {
                (StatusCode::UNAUTHORIZED, err.to_string()).into_response()
            }
            err @ Self::Api(ApiError::UnsupportedMediaType) => {
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, err.to_string()).into_response()
            }
            err => {
                error!(error = ?err, "Error occurred in handler");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
