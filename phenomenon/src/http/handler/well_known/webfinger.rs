use crate::{
    db::model::account,
    error::Result,
    state::Zustand,
    webfinger::{Link, Resource},
};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WebfingerQuery {
    resource: String,
}

pub async fn get(
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

    let Some(account) = account::Entity::find()
        .filter(
            account::Column::Username.eq(username)
                .and(account::Column::Domain.is_null()),
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
