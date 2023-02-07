use crate::state::Zustand;
use axum::{debug_handler, extract::State, routing, Json, Router};
use kitsune_type::nodeinfo::two_one::TwoOne;

#[debug_handler]
async fn get(State(state): State<Zustand>) -> Json<TwoOne> {
    todo!();
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
