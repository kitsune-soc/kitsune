use crate::state::AppState;
use axum::Router;

pub mod media;
pub mod search;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/media", media::routes())
        .nest("/search", search::routes())
}
