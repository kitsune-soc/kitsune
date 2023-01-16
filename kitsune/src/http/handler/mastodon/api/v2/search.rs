use crate::{error::Result, state::Zustand};
use axum::{debug_handler, extract::State, routing, Json, Router};
use axum_extra::extract::Query;
use kitsune_search_proto::{common::SearchIndex, search::SearchResponse};
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
    #[serde(default)]
    resolve: bool,
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
) -> Result<Json<SearchResponse>> {
    let response = state
        .search_service
        .search(SearchIndex::Post, query.query, None)
        .await?;

    Ok(Json(response))
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
