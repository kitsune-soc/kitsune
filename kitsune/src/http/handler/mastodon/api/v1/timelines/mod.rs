use crate::state::AppState;
use axum::{routing, Router};

pub mod home;
pub mod public;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/home", routing::get(home::get))
        .route("/public", routing::get(public::get))
}
