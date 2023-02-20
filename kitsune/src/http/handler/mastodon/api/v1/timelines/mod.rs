use crate::state::Zustand;
use axum::{routing, Router};

mod home;
mod public;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/home", routing::get(home::get))
        .route("/public", routing::get(public::get))
}
