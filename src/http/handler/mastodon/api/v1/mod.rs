use crate::state::State;
use axum::{
    routing::{get, post},
    Router,
};

mod accounts;
mod apps;
mod instance;

pub fn routes() -> Router<State> {
    Router::new()
        .route("/apps", post(apps::post))
        .route("/instance", get(instance::get))
        .nest("/accounts", accounts::routes())
}
