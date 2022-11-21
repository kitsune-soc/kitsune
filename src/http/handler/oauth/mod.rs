//!
//! Standard-compliant OAuth2 flows
//! 
//! Implemented in accordance to [RFC 6749](https://www.rfc-editor.org/rfc/rfc6749)
//! 

use axum::{Router, routing::get};

pub mod authorize;

pub fn routes() -> Router {
    Router::new().route("/authorize", get(authorize::get))
}
