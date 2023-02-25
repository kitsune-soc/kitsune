use crate::{
    config::Configuration,
    search::{schema::PrepareQuery, SearchIndex},
};
use autometrics::autometrics;
use kitsune_search_proto::{
    common::SearchIndex as GrpcSearchIndex,
    search::{search_server::Search, SearchRequest, SearchResponse, SearchResult},
};
use std::ops::Bound;
use tantivy::{collector::TopDocs, IndexReader};
use tonic::{async_trait, Request, Response, Status};

/// Search service
pub struct SearchService {
    /// Reader of the account index
    pub account: IndexReader,

    /// Reader of the post index
    pub post: IndexReader,
}

#[async_trait]
#[autometrics(track_concurrency)]
impl Search for SearchService {
    async fn search(&self, req: Request<SearchRequest>) -> tonic::Result<Response<SearchResponse>> {
        let config = req.extensions().get::<Configuration>().unwrap();
        let index = req.extensions().get::<SearchIndex>().unwrap();

        let bounds = (
            req.get_ref()
                .min_id
                .as_deref()
                .map_or(Bound::Unbounded, Bound::Included),
            req.get_ref()
                .max_id
                .as_deref()
                .map_or(Bound::Unbounded, Bound::Included),
        );
        let (query, searcher, id_field, indexed_at_field) = match req.get_ref().index() {
            GrpcSearchIndex::Account => (
                index.schemas.account.prepare_query(
                    &req.get_ref().query,
                    bounds,
                    config.levenshtein_distance,
                ),
                self.account.searcher(),
                index.schemas.account.id,
                index.schemas.account.indexed_at,
            ),
            GrpcSearchIndex::Post => (
                index.schemas.post.prepare_query(
                    &req.get_ref().query,
                    bounds,
                    config.levenshtein_distance,
                ),
                self.post.searcher(),
                index.schemas.post.id,
                index.schemas.post.indexed_at,
            ),
        };

        let top_docs_collector = TopDocs::with_limit(req.get_ref().max_results as usize)
            .and_offset(req.get_ref().offset as usize)
            .order_by_fast_field::<u64>(indexed_at_field);
        let results = searcher
            .search(&query, &top_docs_collector)
            .map_err(|e| Status::internal(e.to_string()))?;

        let documents = results
            .into_iter()
            .map(|(_score, addr)| {
                searcher.doc(addr).map(|doc| {
                    let id = doc
                        .get_first(id_field)
                        .unwrap()
                        .as_bytes()
                        .unwrap()
                        .to_vec();

                    SearchResult { id }
                })
            })
            .collect::<Result<_, _>>()
            .map_err(|err| Status::internal(err.to_string()))?;

        increment_counter!("served_search_requests");

        Ok(Response::new(SearchResponse { results: documents }))
    }
}
