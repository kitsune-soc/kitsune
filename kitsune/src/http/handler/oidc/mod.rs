use crate::state::Zustand;
use axum::{routing, Router};

pub mod callback;

pub fn routes() -> Router<Zustand> {
    Router::new().route("/callback", routing::get(callback::get))
}
