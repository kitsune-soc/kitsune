use crate::grpc::proto::search::{search_server::Search, SearchRequest, SearchResponse};
use tonic::{async_trait, Request, Response};

pub struct SearchService;

#[async_trait]
impl Search for SearchService {
    async fn search(&self, req: Request<SearchRequest>) -> tonic::Result<Response<SearchResponse>> {
        todo!();
    }
}
