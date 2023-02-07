use crate::{error::Result, state::Zustand};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use http::StatusCode;
use kitsune_db::entity::{accounts, prelude::Accounts};
use kitsune_type::webfinger::{Link, Resource};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

#[derive(Deserialize)]
struct WebfingerQuery {
    resource: String,
}

async fn get(
    State(state): State<Zustand>,
    Query(query): Query<WebfingerQuery>,
) -> Result<Response> {
    let username_at_instance = query.resource.trim_start_matches("acct:");
    let Some((username, instance)) = username_at_instance.split_once('@') else {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    };

    if instance != state.config.domain {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let Some(account) = Accounts::find()
        .filter(
            accounts::Column::Username.eq(username)
                .and(accounts::Column::Domain.is_null()),
        )
        .one(&state.db_conn)
        .await?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(Resource {
        subject: query.resource,
        aliases: vec![account.url.clone()],
        links: vec![Link {
            rel: "self".into(),
            href: Some(account.url),
        }],
    })
    .into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
