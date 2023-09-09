use axum::{routing, Router};
use kitsune_core::state::Zustand;

pub mod callback;

pub fn routes() -> Router<Zustand> {
    Router::new().route("/callback", routing::get(callback::get))
}
