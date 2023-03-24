use crate::state::Zustand;
use axum::Router;

pub mod api;

pub fn routes() -> Router<Zustand> {
    Router::new().nest("/api", api::routes())
}
