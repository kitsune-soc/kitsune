use crate::{
    error::Result,
    service::{instance::InstanceService, url::UrlService},
    state::Zustand,
};
use axum::{extract::State, routing, Json, Router};
use kitsune_type::mastodon::{
    instance::{Stats, Urls},
    Instance,
};

async fn get(
    State(instance_service): State<InstanceService>,
    State(url_service): State<UrlService>,
) -> Result<Json<Instance>> {
    let user_count = instance_service.user_count().await?;
    let domain_count = instance_service.known_instances().await?;

    Ok(Json(Instance {
        uri: url_service.domain().into(),
        title: instance_service.name().into(),
        short_description: instance_service.description().into(),
        description: String::new(),
        email: String::new(),
        version: env!("CARGO_PKG_VERSION").into(),
        urls: Urls {
            streaming_api: String::new(),
        },
        stats: Stats {
            user_count,
            domain_count,
            status_count: 0,
        },
    }))
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
