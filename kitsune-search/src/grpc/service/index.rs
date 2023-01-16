use crate::search::SearchIndex;
use futures_util::TryStreamExt;
use kitsune_search_proto::{
    common::SearchIndex as GrpcSearchIndex,
    index::{
        add_index_request::IndexData, index_server::Index, AddIndexRequest, AddIndexResponse,
        RemoveIndexRequest, RemoveIndexResponse, ResetRequest, ResetResponse,
    },
};
use tantivy::{Document, IndexWriter, Term};
use tokio::sync::RwLock;
use tonic::{async_trait, Request, Response, Status, Streaming};

/// Maximum amount of documents that get concurrently indexed
const MAX_CONCURRENT_INDEXING: usize = 50;

pub struct IndexService {
    pub account: RwLock<IndexWriter>,
    pub post: RwLock<IndexWriter>,
}

impl IndexService {
    async fn add_document(&self, req: AddIndexRequest, index: &SearchIndex) -> tonic::Result<()> {
        let (writer, document) = match req.index_data {
            Some(IndexData::Account(data)) => {
                let account_schema = &index.schemas.account;
                let mut document = Document::new();
                document.add_bytes(account_schema.id, data.id);
                document.add_text(account_schema.username, data.username);

                if let Some(display_name) = data.display_name {
                    document.add_text(account_schema.display_name, display_name);
                }
                if let Some(description) = data.description {
                    document.add_text(account_schema.description, description);
                }

                (self.account.read().await, document)
            }
            Some(IndexData::Post(data)) => {
                let post_schema = &index.schemas.post;
                let mut document = Document::new();
                document.add_bytes(post_schema.id, data.id);
                document.add_text(post_schema.content, data.content);

                if let Some(subject) = data.subject {
                    document.add_text(post_schema.subject, subject);
                }

                (self.post.read().await, document)
            }
            None => return Err(Status::invalid_argument("missing index data")),
        };

        writer
            .add_document(document)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(())
    }

    async fn commit_all(&self) -> tonic::Result<()> {
        self.account
            .write()
            .await
            .prepare_commit()
            .unwrap()
            .commit_future()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        self.post
            .write()
            .await
            .prepare_commit()
            .unwrap()
            .commit_future()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(())
    }

    async fn delete_document(
        &self,
        req: RemoveIndexRequest,
        index: &SearchIndex,
    ) -> tonic::Result<()> {
        let (writer, id_field) = match req.index() {
            GrpcSearchIndex::Account => (self.account.read().await, index.schemas.account.id),
            GrpcSearchIndex::Post => (self.post.read().await, index.schemas.post.id),
        };

        let term = Term::from_field_bytes(id_field, &req.id);
        writer.delete_term(term);

        Ok(())
    }
}

#[async_trait]
impl Index for IndexService {
    async fn add(
        &self,
        mut req: Request<Streaming<AddIndexRequest>>,
    ) -> tonic::Result<Response<AddIndexResponse>> {
        let index = req.extensions().get::<SearchIndex>().unwrap().clone();

        req.get_mut()
            .map_ok(|req| self.add_document(req, &index))
            .try_buffer_unordered(MAX_CONCURRENT_INDEXING)
            .try_collect::<()>()
            .await?;

        self.commit_all().await?;

        Ok(Response::new(AddIndexResponse {}))
    }

    async fn remove(
        &self,
        mut req: Request<Streaming<RemoveIndexRequest>>,
    ) -> tonic::Result<Response<RemoveIndexResponse>> {
        let index = req.extensions().get::<SearchIndex>().unwrap().clone();

        req.get_mut()
            .map_ok(|req| self.delete_document(req, &index))
            .try_buffer_unordered(MAX_CONCURRENT_INDEXING)
            .try_collect()
            .await?;

        self.commit_all().await?;

        Ok(Response::new(RemoveIndexResponse {}))
    }

    async fn reset(&self, req: Request<ResetRequest>) -> tonic::Result<Response<ResetResponse>> {
        let mut writer = match req.get_ref().index() {
            GrpcSearchIndex::Account => self.account.write().await,
            GrpcSearchIndex::Post => self.post.write().await,
        };

        writer
            .delete_all_documents()
            .map_err(|e| Status::internal(e.to_string()))?;

        writer
            .prepare_commit()
            .unwrap()
            .commit_future()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ResetResponse {}))
    }
}
