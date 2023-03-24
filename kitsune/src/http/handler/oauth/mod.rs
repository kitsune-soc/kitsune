//!
//! Standard-compliant OAuth2 flows
//!
//! Implemented (somewhat) in accordance to [RFC 6749](https://www.rfc-editor.org/rfc/rfc6749)
//!

use crate::state::Zustand;
use axum::{
    routing::{get, post},
    Router,
};
use chrono::Duration;
use once_cell::sync::Lazy;

pub mod authorize;
pub mod token;

static TOKEN_VALID_DURATION: Lazy<Duration> = Lazy::new(|| Duration::hours(1));

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/authorize", get(authorize::get).post(authorize::post))
        .route("/token", post(token::post))
}
