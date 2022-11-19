use axum::{routing::post, Router};

pub mod register;

pub fn routes() -> Router {
    Router::new().route("/register", post(register::post))
}
