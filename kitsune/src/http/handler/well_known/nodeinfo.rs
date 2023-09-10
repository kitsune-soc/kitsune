use crate::state::AppState;
use axum::{extract::State, routing, Json, Router};
use kitsune_core::service::url::UrlService;
use kitsune_type::nodeinfo::well_known::{Link, WellKnown};

#[allow(clippy::unused_async)]
#[utoipa::path(
    get,
    path = "/.well-known/nodeinfo",
    responses(
        (status = 200, description = "Response with the location of the nodeinfo endpoints", body = WellKnown)
    )
)]
async fn get(State(url_service): State<UrlService>) -> Json<WellKnown> {
    let href = format!("{}/nodeinfo/2.1", url_service.base_url());

    Json(WellKnown {
        links: vec![Link {
            rel: kitsune_type::nodeinfo::well_known::Rel::TwoOne,
            href,
        }],
    })
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/", routing::get(get))
}
