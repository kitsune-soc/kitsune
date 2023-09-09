use axum::{routing, Router};
use kitsune_core::state::Zustand;

pub mod home;
pub mod public;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/home", routing::get(home::get))
        .route("/public", routing::get(public::get))
}
