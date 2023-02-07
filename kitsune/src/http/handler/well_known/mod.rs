use crate::state::Zustand;
use axum::Router;

mod nodeinfo;
mod webfinger;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .nest("/nodeinfo", nodeinfo::routes())
        .nest("/webfinger", webfinger::routes())
}
