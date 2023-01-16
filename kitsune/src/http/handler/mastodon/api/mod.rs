use crate::state::Zustand;
use axum::Router;

mod v1;
mod v2;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .nest("/v1", v1::routes())
        .nest("/v2", v2::routes())
}
