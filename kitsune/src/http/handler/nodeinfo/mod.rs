use crate::state::AppState;
use axum::Router;

pub mod two_one;

pub fn routes() -> Router<AppState> {
    Router::new().nest("/2.1", two_one::routes())
}
