use crate::{
    error::Result,
    http::extractor::MastodonAuthExtractor,
    service::search::{ArcSearchService, SearchService},
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::State,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_extra::extract::Query;
use http::StatusCode;
use kitsune_db::entity::prelude::{Accounts, Posts};
use kitsune_search_proto::common::SearchIndex;
use kitsune_type::mastodon::SearchResult;
use sea_orm::EntityTrait;
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

#[debug_handler(state = Zustand)]
async fn get(
    State(state): State<Zustand>,
    State(search): State<ArcSearchService>,
    _: MastodonAuthExtractor,
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

    let mut search_result = SearchResult::default();
    for index in indices {
        let results = search
            .search(
                index,
                query.query.clone(),
                query.limit,
                query.offset,
                query.min_id,
                query.max_id,
            )
            .await?;

        for result in results {
            let id = Uuid::from_bytes(
                result
                    .id
                    .try_into()
                    .expect("[Bug] Non-UUID indexed in search index"),
            );

            match index {
                SearchIndex::Account => {
                    let account = Accounts::find_by_id(id)
                        .one(&state.db_conn)
                        .await?
                        .expect("[Bug] Account indexed in search not in database");

                    search_result
                        .accounts
                        .push(state.mastodon_mapper.map(account).await?);
                }
                SearchIndex::Post => {
                    let post = Posts::find_by_id(id)
                        .one(&state.db_conn)
                        .await?
                        .expect("[Bug] Post indexed in search not in database");

                    search_result
                        .statuses
                        .push(state.mastodon_mapper.map(post).await?);
                }
            }
        }
    }

    Ok(Json(search_result).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
