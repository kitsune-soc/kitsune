use crate::{
    config::Configuration,
    grpc::proto::{
        common::SearchIndex as GrpcSearchIndex,
        search::{search_server::Search, SearchRequest, SearchResponse, SearchResult},
    },
    search::{schema::PrepareQuery, SearchIndex},
};
use tantivy::{
    collector::{Count, TopDocs},
    IndexReader,
};
use tonic::{async_trait, Request, Response, Status};

/// Results per page
const PER_PAGE: usize = 20;

pub struct SearchService {
    pub account: IndexReader,
    pub post: IndexReader,
}

#[async_trait]
impl Search for SearchService {
    async fn search(&self, req: Request<SearchRequest>) -> tonic::Result<Response<SearchResponse>> {
        let config = req.extensions().get::<Configuration>().unwrap();
        let index = req.extensions().get::<SearchIndex>().unwrap();

        let (query, searcher, id_field) = match req.get_ref().index() {
            GrpcSearchIndex::Account => (
                index
                    .schemas
                    .account
                    .prepare_query(&req.get_ref().query, config.levenshtein_distance),
                self.account.searcher(),
                index.schemas.account.id,
            ),
            GrpcSearchIndex::Post => (
                index
                    .schemas
                    .post
                    .prepare_query(&req.get_ref().query, config.levenshtein_distance),
                self.post.searcher(),
                index.schemas.post.id,
            ),
        };

        let top_docs_collector =
            TopDocs::with_limit(PER_PAGE).and_offset((req.get_ref().page as usize) * PER_PAGE);

        let (count, results) = match searcher.search(&query, &(Count, top_docs_collector)) {
            Ok(result) => result,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        let documents = match results
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
            .collect()
        {
            Ok(docs) => docs,
            Err(err) => return Err(Status::internal(err.to_string())),
        };

        Ok(Response::new(SearchResponse {
            result: documents,
            page: req.get_ref().page,
            total_pages: crate::util::div_ceil(count, PER_PAGE) as u64,
        }))
    }
}
