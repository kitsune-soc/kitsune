use axum::Router;
use kitsune_core::state::Zustand;

pub mod nodeinfo;
pub mod webfinger;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .nest("/nodeinfo", nodeinfo::routes())
        .nest("/webfinger", webfinger::routes())
}
