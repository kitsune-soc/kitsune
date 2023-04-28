use crate::{error::Result, service::url::UrlService, state::Zustand};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use http::StatusCode;
use kitsune_db::entity::{accounts, prelude::Accounts};
use kitsune_type::webfinger::{Link, Resource};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
struct WebfingerQuery {
    resource: String,
}

#[utoipa::path(
    get,
    path = "/.well-known/webfinger",
    params(WebfingerQuery),
    responses(
        (status = 200, description = "Response with the location of the user's profile", body = Resource),
        (status = StatusCode::NOT_FOUND, description = "The service doesn't know this user"),
    )
)]
async fn get(
    State(db_conn): State<DatabaseConnection>,
    State(url_service): State<UrlService>,
    Query(query): Query<WebfingerQuery>,
) -> Result<Response> {
    let username_at_instance = query.resource.trim_start_matches("acct:");
    let Some((username, instance)) = username_at_instance.split_once('@') else {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    };

    if instance != url_service.domain() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let Some(account) = Accounts::find()
        .filter(
            accounts::Column::Username.eq(username)
                .and(accounts::Column::Local.eq(true)),
        )
        .one(&db_conn)
        .await?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let account_url = url_service.user_url(account.id);

    Ok(Json(Resource {
        subject: query.resource,
        aliases: vec![account_url.clone()],
        links: vec![Link {
            rel: "self".into(),
            r#type: Some("application/activity+json".into()),
            href: Some(account_url),
        }],
    })
    .into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
