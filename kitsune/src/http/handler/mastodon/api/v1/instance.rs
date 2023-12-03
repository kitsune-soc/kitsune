use crate::{error::Result, state::Zustand};
use axum::{extract::State, routing, Json, Router};
use kitsune_core::consts::VERSION;
use kitsune_service::instance::InstanceService;
use kitsune_type::mastodon::{
    instance::{Stats, Urls},
    Instance,
};
use kitsune_url::UrlService;

#[utoipa::path(
    get,
    path = "/api/v1/instance",
    responses(
        (status = 200, description = "Instance metadata", body = Instance),
    ),
)]
async fn get(
    State(instance_service): State<InstanceService>,
    State(url_service): State<UrlService>,
) -> Result<Json<Instance>> {
    let status_count = instance_service.local_post_count().await?;
    let user_count = instance_service.user_count().await?;
    let domain_count = instance_service.known_instances().await?;

    Ok(Json(Instance {
        uri: url_service.webfinger_domain().into(),
        title: instance_service.name().into(),
        short_description: instance_service.description().into(),
        description: String::new(),
        max_toot_chars: instance_service.character_limit(),
        email: String::new(),
        version: VERSION.into(),
        urls: Urls {
            streaming_api: String::new(),
        },
        stats: Stats {
            user_count,
            status_count,
            domain_count,
        },
    }))
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
