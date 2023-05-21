use crate::{error::Result, state::Zustand};
use axum::{debug_handler, extract::State, routing, Json, Router};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::schema::{posts, users};
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
    let mut db_conn = state.db_conn.get().await?;

    let total_fut = users::table.count().get_result::<i64>(&mut db_conn);
    let local_posts_fut = posts::table
        .filter(posts::is_local.eq(true))
        .count()
        .get_result::<i64>(&mut db_conn);

    let (total, local_posts) = tokio::try_join!(total_fut, local_posts_fut)?;

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
                total: total as u64,
                active_halfyear: None,
                active_month: None,
            },
            local_comments: None,
            local_posts: local_posts as u64,
        },
        metadata: Value::Null,
    }))
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
