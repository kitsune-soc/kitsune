use crate::state::AppState;
use axum::Router;

pub mod api;

pub fn routes() -> Router<AppState> {
    Router::new().nest("/api", api::routes())
}
