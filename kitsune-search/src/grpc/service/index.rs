use crate::{
    grpc::proto::{
        common::SearchIndex as GrpcSearchIndex,
        index::{
            add_index_request::IndexData, index_server::Index, AddIndexRequest, AddIndexResponse,
            RemoveIndexRequest, RemoveIndexResponse, ResetRequest, ResetResponse,
        },
    },
    search::SearchIndex,
};
use tantivy::{Document, IndexWriter, Term};
use tokio::sync::Mutex;
use tonic::{async_trait, Request, Response, Status};

pub struct IndexService {
    pub account: Mutex<IndexWriter>,
    pub post: Mutex<IndexWriter>,
}

#[async_trait]
impl Index for IndexService {
    async fn add(
        &self,
        req: Request<AddIndexRequest>,
    ) -> tonic::Result<Response<AddIndexResponse>> {
        let index = req.extensions().get::<SearchIndex>().unwrap();

        let (mut writer, document) = match &req.get_ref().index_data {
            Some(IndexData::Account(data)) => {
                let account_schema = &index.schemas.account;
                let mut document = Document::new();
                document.add_bytes(account_schema.id, data.id.as_slice());
                document.add_text(account_schema.username, &data.username);

                if let Some(ref display_name) = data.display_name {
                    document.add_text(account_schema.display_name, display_name);
                }
                if let Some(ref description) = data.description {
                    document.add_text(account_schema.description, description);
                }

                (self.account.lock().await, document)
            }
            Some(IndexData::Post(data)) => {
                let post_schema = &index.schemas.post;
                let mut document = Document::new();
                document.add_bytes(post_schema.id, data.id.as_slice());
                document.add_text(post_schema.content, &data.content);

                if let Some(ref subject) = data.subject {
                    document.add_text(post_schema.subject, subject);
                }

                (self.post.lock().await, document)
            }
            None => return Err(Status::invalid_argument("missing index data")),
        };

        if let Err(e) = writer.add_document(document) {
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
        let (mut writer, id_field) = match req.get_ref().index() {
            GrpcSearchIndex::Account => (self.account.lock().await, index.schemas.account.id),
            GrpcSearchIndex::Post => (self.post.lock().await, index.schemas.post.id),
        };

        let term = Term::from_field_bytes(id_field, &req.get_ref().id);
        writer.delete_term(term);

        if let Err(e) = writer.prepare_commit().unwrap().commit_future().await {
            return Err(Status::internal(e.to_string()));
        }

        Ok(Response::new(RemoveIndexResponse {}))
    }

    async fn reset(&self, req: Request<ResetRequest>) -> tonic::Result<Response<ResetResponse>> {
        let mut writer = match req.get_ref().index() {
            GrpcSearchIndex::Account => self.account.lock().await,
            GrpcSearchIndex::Post => self.post.lock().await,
        };

        if let Err(e) = writer.delete_all_documents() {
            return Err(Status::internal(e.to_string()));
        }
        if let Err(e) = writer.prepare_commit().unwrap().commit_future().await {
            return Err(Status::internal(e.to_string()));
        }

        Ok(Response::new(ResetResponse {}))
    }
}
