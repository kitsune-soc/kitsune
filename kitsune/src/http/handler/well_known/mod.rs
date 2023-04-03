use crate::state::Zustand;
use axum::Router;

pub mod nodeinfo;
pub mod webfinger;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .nest("/nodeinfo", nodeinfo::routes())
        .nest("/webfinger", webfinger::routes())
}
