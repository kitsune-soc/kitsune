use crate::state::Zustand;
use axum::{routing, Router};

mod public;

pub fn routes() -> Router<Zustand> {
    Router::new().route("/public", routing::get(public::get))
}
