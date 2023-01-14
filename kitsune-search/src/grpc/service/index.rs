use crate::{
    grpc::proto::index::{
        index_server::Index, AddIndexRequest, AddIndexResponse, RemoveIndexRequest,
        RemoveIndexResponse, ResetRequest, ResetResponse,
    },
    search::SearchIndex,
};
use tantivy::{doc, IndexWriter, Term};
use tokio::sync::Mutex;
use tonic::{async_trait, Request, Response, Status};

pub struct IndexService {
    pub writer: Mutex<IndexWriter>,
}

#[async_trait]
impl Index for IndexService {
    async fn add(
        &self,
        req: Request<AddIndexRequest>,
    ) -> tonic::Result<Response<AddIndexResponse>> {
        let index = req.extensions().get::<SearchIndex>().unwrap();
        let mut writer = self.writer.lock().await;

        let doc = doc! {
            index.schema.id => req.get_ref().id.as_slice(),
            index.schema.data => req.get_ref().data.as_str(),
        };

        if let Err(e) = writer.add_document(doc) {
            return Err(Status::internal(e.to_string()));
        }

        if let Err(e) = writer.prepare_commit().unwrap().commit_future().await {
            return Err(Status::internal(e.to_string()));
        }

        Ok(Response::new(AddIndexResponse {}))
    }

    async fn remove(
        &self,
        req: Request<RemoveIndexRequest>,
    ) -> tonic::Result<Response<RemoveIndexResponse>> {
        let index = req.extensions().get::<SearchIndex>().unwrap();
        let mut writer = self.writer.lock().await;

        let term = Term::from_field_bytes(index.schema.id, &req.get_ref().id);
        writer.delete_term(term);

        if let Err(e) = writer.prepare_commit().unwrap().commit_future().await {
            return Err(Status::internal(e.to_string()));
        }

        Ok(Response::new(RemoveIndexResponse {}))
    }

    async fn reset(&self, _req: Request<ResetRequest>) -> tonic::Result<Response<ResetResponse>> {
        let mut writer = self.writer.lock().await;

        if let Err(e) = writer.delete_all_documents() {
            return Err(Status::internal(e.to_string()));
        }
        if let Err(e) = writer.prepare_commit().unwrap().commit_future().await {
            return Err(Status::internal(e.to_string()));
        }

        Ok(Response::new(ResetResponse {}))
    }
}
