use crate::{error::Result, search::SearchService, state::Zustand};
use axum::{
    debug_handler,
    extract::State,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_extra::extract::Query;
use http::StatusCode;
use kitsune_search_proto::common::SearchIndex;
use serde::Deserialize;
use uuid::Uuid;

fn default_page_limit() -> u64 {
    40
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchType {
    Accounts,
    Hashtags,
    Statuses,
}

#[derive(Deserialize)]
struct SearchQuery {
    #[serde(rename = "q")]
    query: String,
    r#type: Option<SearchType>,
    /*#[serde(default)]
    resolve: bool,*/
    max_id: Option<Uuid>,
    min_id: Option<Uuid>,
    #[serde(default = "default_page_limit")]
    limit: u64,
    #[serde(default)]
    offset: u64,
}

#[debug_handler]
async fn get(
    State(mut state): State<Zustand>,
    Query(query): Query<SearchQuery>,
) -> Result<Response> {
    let indices = if let Some(r#type) = query.r#type {
        let index = match r#type {
            SearchType::Accounts => SearchIndex::Account,
            SearchType::Statuses => SearchIndex::Post,
            SearchType::Hashtags => return Ok(StatusCode::BAD_REQUEST.into_response()),
        };

        vec![index]
    } else {
        vec![SearchIndex::Account, SearchIndex::Post]
    };

    let mut results = Vec::new();
    for index in indices {
        let mut response = state
            .search_service
            .search(
                index,
                query.query.clone(),
                query.limit,
                query.offset,
                query.min_id,
                query.max_id,
            )
            .await?;

        results.append(&mut response.result);
    }

    Ok(Json(results).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
