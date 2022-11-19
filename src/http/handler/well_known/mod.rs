use axum::{routing::get, Router};

pub mod webfinger;

pub fn routes() -> Router {
    Router::new().route("/webfinger", get(webfinger::get))
}
