//!
//! Standard-compliant OAuth2 flows
//!

use crate::state::Zustand;
use axum::{
    routing::{get, post},
    Router,
};

pub mod authorize;
pub mod token;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/authorize", get(authorize::get).post(authorize::post))
        .route("/token", post(token::post))
}
