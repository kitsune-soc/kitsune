use axum::Router;

pub mod v1;

pub fn routes() -> Router {
    Router::new().nest("/v1", self::v1::routes())
}
