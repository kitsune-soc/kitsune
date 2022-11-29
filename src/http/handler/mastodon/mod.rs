use crate::state::Zustand;
use axum::Router;

mod api;

pub fn routes() -> Router<Zustand> {
    Router::new().nest("/api", api::routes())
}
