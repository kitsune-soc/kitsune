use axum::{
    routing::{get, post},
    Router,
};

mod apps;
mod instance;

pub fn routes() -> Router {
    Router::new()
        .route("/apps", post(apps::post))
        .route("/instance", get(instance::get))
}
