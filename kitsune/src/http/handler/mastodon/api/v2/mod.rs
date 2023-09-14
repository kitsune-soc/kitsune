use crate::state::Zustand;
use axum::Router;

pub mod media;
pub mod search;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .nest("/media", media::routes())
        .nest("/search", search::routes())
}
