use crate::state::State;
use axum::Router;

mod v1;

pub fn routes() -> Router<State> {
    Router::new().nest("/v1", v1::routes())
}
