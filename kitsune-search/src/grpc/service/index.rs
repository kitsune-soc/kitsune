use crate::grpc::proto::index::{
    index_server::Index, AddIndexRequest, AddIndexResponse, RemoveIndexRequest, RemoveIndexResponse,
};
use tonic::{async_trait, Request, Response};

pub struct IndexService;

#[async_trait]
impl Index for IndexService {
    async fn add(
        &self,
        req: Request<AddIndexRequest>,
    ) -> tonic::Result<Response<AddIndexResponse>> {
        todo!();
    }

    async fn remove(
        &self,
        req: Request<RemoveIndexRequest>,
    ) -> tonic::Result<Response<RemoveIndexResponse>> {
        todo!();
    }
}
