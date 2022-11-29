use crate::state::Zustand;
use axum::Router;

mod v1;

pub fn routes() -> Router<Zustand> {
    Router::new().nest("/v1", v1::routes())
}
