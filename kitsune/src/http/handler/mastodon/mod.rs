use axum::Router;
use kitsune_core::state::Zustand;

pub mod api;

pub fn routes() -> Router<Zustand> {
    Router::new().nest("/api", api::routes())
}
