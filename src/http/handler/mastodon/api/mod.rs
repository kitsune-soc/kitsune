use axum::Router;

mod v1;

pub fn routes() -> Router {
    Router::new().nest("/v1", v1::routes())
}
