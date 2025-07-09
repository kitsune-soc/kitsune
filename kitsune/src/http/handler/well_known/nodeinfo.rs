use axum::{Json, extract::State};
use kitsune_type::nodeinfo::well_known::{Link, WellKnown};
use kitsune_url::UrlService;

#[allow(clippy::unused_async)]
pub async fn get(State(url_service): State<UrlService>) -> Json<WellKnown> {
    let href = format!("{}/nodeinfo/2.1", url_service.base_url());

    Json(WellKnown {
        links: vec![Link {
            rel: kitsune_type::nodeinfo::well_known::Rel::TwoOne,
            href,
        }],
    })
}
