//!
//! Standard-compliant OAuth2 flows
//!

use axum::{
    routing::{get, post},
    Router,
};
use kitsune_core::state::Zustand;

pub mod authorize;
pub mod token;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/authorize", get(authorize::get).post(authorize::post))
        .route("/token", post(token::post))
}
