use crate::{
    grpc::proto::search::{search_server::Search, SearchRequest, SearchResponse, SearchResult},
    search::SearchIndex,
};
use tantivy::{collector::TopDocs, query::FuzzyTermQuery, IndexReader, Term};
use tonic::{async_trait, Request, Response, Status};

pub struct SearchService {
    pub reader: IndexReader,
}

#[async_trait]
impl Search for SearchService {
    async fn search(&self, req: Request<SearchRequest>) -> tonic::Result<Response<SearchResponse>> {
        let index = req.extensions().get::<SearchIndex>().unwrap();
        let searcher = self.reader.searcher();

        let term = Term::from_field_text(index.schema.data, &req.get_ref().query);
        let query = FuzzyTermQuery::new(term, 2, true);
        let result = match searcher.search(
            &query,
            &TopDocs::with_limit(20).and_offset(req.get_ref().offset as usize),
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
                        doc.get_first(index.schema.id)
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
