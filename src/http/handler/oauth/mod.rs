//!
//! Standard-compliant OAuth2 flows
//!
//! Implemented (somewhat) in accordance to [RFC 6749](https://www.rfc-editor.org/rfc/rfc6749)
//!
//! TODO: Implement client credentials grant
//!

use axum::{
    routing::{get, post},
    Router,
};
use chrono::Duration;
use once_cell::sync::Lazy;

mod authorize;
mod token;

static TOKEN_VALID_DURATION: Lazy<Duration> = Lazy::new(|| Duration::hours(1));

pub fn routes() -> Router {
    Router::new()
        .route("/authorize", get(authorize::get).post(authorize::post))
        .route("/token", post(token::post))
}
