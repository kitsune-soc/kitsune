use crate::state::Zustand;
use axum::{
    routing::{get, post},
    Router,
};

mod accounts;
mod apps;
mod instance;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/apps", post(apps::post))
        .route("/instance", get(instance::get))
        .nest("/accounts", accounts::routes())
}
