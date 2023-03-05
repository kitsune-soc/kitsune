use crate::state::Zustand;
use axum::{routing, Router};

use crate::http::handler::mastodon::api::v1::media::post;

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::post(post))
}
