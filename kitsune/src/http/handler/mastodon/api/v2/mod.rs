use crate::state::Zustand;
use axum::Router;

mod search;

pub fn routes() -> Router<Zustand> {
    Router::new().nest("/search", search::routes())
}
