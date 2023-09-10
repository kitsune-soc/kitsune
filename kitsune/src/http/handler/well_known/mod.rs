use crate::state::AppState;
use axum::Router;

pub mod nodeinfo;
pub mod webfinger;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/nodeinfo", nodeinfo::routes())
        .nest("/webfinger", webfinger::routes())
}
