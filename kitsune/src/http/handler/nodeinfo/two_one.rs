use crate::{error::Result, state::Zustand};
use axum::{debug_handler, extract::State, routing, Json, Router};
use kitsune_type::nodeinfo::two_one::{
    Protocol, Services, Software, TwoOne, Usage, UsageUsers, Version,
};
use serde_json::Value;

#[debug_handler]
#[utoipa::path(
    get,
    path = "/nodeinfo/2.1",
    responses(
        (status = 200, description = "Get response following the Nodeinfo 2.1 schema", body = TwoOne)
    ),
)]
async fn get(State(state): State<Zustand>) -> Result<Json<TwoOne>> {
    let total = Users::find().count(&state.db_conn).await?;
    let local_posts = Posts::find()
        .filter(posts::Column::IsLocal.eq(true))
        .count(&state.db_conn)
        .await?;

    Ok(Json(TwoOne {
        version: Version::TwoOne,
        software: Software {
            name: env!("CARGO_PKG_NAME").into(),
            version: env!("CARGO_PKG_VERSION").into(),
            repository: env!("CARGO_PKG_REPOSITORY").into(),
            homepage: Some(env!("CARGO_PKG_HOMEPAGE").into()),
        },
        protocols: vec![Protocol::ActivityPub],
        services: Services {
            inbound: vec![],
            outbound: vec![],
        },
        open_registrations: true,
        usage: Usage {
            users: UsageUsers {
                total,
                active_halfyear: None,
                active_month: None,
            },
            local_comments: None,
            local_posts,
        },
        metadata: Value::Null,
    }))
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
