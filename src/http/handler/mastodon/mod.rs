use crate::state::State;
use axum::Router;

mod api;

pub fn routes() -> Router<State> {
    Router::new().nest("/api", api::routes())
}
