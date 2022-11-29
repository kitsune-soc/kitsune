use crate::state::Zustand;
use axum::{routing::get, Router};

pub mod webfinger;

pub fn routes() -> Router<Zustand> {
    Router::new().route("/webfinger", get(webfinger::get))
}
