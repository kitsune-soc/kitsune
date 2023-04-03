use crate::state::Zustand;
use axum::Router;

pub mod two_one;

pub fn routes() -> Router<Zustand> {
    Router::new().nest("/2.1", two_one::routes())
}
