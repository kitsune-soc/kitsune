//!
//! Standard-compliant OAuth2 flows
//!

use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub mod authorize;
pub mod token;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/authorize", get(authorize::get).post(authorize::post))
        .route("/token", post(token::post))
}
