use crate::state::Zustand;
use axum::Router;

mod media;
mod search;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .nest("/media", media::routes())
        .nest("/search", search::routes())
}
