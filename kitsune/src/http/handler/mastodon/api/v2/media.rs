use crate::http::handler::mastodon::api::v1::media::{get, post, put};
use crate::state::AppState;
use axum::{routing, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", routing::post(post))
        .route("/:id", routing::get(get).put(put))
}
