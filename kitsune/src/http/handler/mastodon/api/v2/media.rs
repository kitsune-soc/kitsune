use crate::{
    http::handler::mastodon::api::v1::media::{get, post, put},
    state::Zustand,
};
use axum::{routing, Router};

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/", routing::post(post))
        .route("/:id", routing::get(get).put(put))
}
