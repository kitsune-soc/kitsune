use crate::state::State;
use axum::{routing::get, Router};

pub mod webfinger;

pub fn routes() -> Router<State> {
    Router::new().route("/webfinger", get(webfinger::get))
}
