use crate::{
    config::Configuration,
    grpc::proto::{
        common::SearchIndex as GrpcSearchIndex,
        search::{search_server::Search, SearchRequest, SearchResponse, SearchResult},
    },
    search::{schema::PrepareQuery, SearchIndex},
};
use tantivy::{collector::TopDocs, IndexReader};
use tonic::{async_trait, Request, Response, Status};

const PAGE_LIMIT: usize = 20;

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

        let result = match searcher.search(
            &query,
            &TopDocs::with_limit(PAGE_LIMIT).and_offset(req.get_ref().offset as usize),
        ) {
            Ok(result) => result,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        let documents = match result
            .into_iter()
            .map(|(_score, addr)| {
                searcher
                    .doc(addr)
                    .map(|doc| {
                        doc.get_first(id_field)
                            .unwrap()
                            .as_bytes()
                            .unwrap()
                            .to_vec()
                    })
                    .map(|id| SearchResult { id })
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(docs) => docs,
            Err(err) => return Err(Status::internal(err.to_string())),
        };

        Ok(Response::new(SearchResponse { result: documents }))
    }
}
