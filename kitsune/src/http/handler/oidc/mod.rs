use crate::state::AppState;
use axum::{routing, Router};

pub mod callback;

pub fn routes() -> Router<AppState> {
    Router::new().route("/callback", routing::get(callback::get))
}
