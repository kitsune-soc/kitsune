use axum::Router;

mod api;

pub fn routes() -> Router {
    Router::new().nest("/api", api::routes())
}
