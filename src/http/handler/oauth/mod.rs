//!
//! Standard-compliant OAuth2 flows
//!
//! Implemented in accordance to [RFC 6749](https://www.rfc-editor.org/rfc/rfc6749)
//!

use axum::{
    routing::{get, post},
    Router,
};

mod authorize;
mod token;

pub fn routes() -> Router {
    Router::new()
        .route("/authorize", get(authorize::get).post(authorize::post))
        .route("/token", post(token::post))
}
