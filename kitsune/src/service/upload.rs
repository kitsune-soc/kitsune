use crate::error::{Error, Result};
use bytes::Bytes;
use derive_builder::Builder;
use futures_util::{Stream, StreamExt};
use kitsune_db::entity::media_attachments;
use kitsune_storage::{BoxError, StorageBackend};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct Upload<S> {
    account_id: Uuid,
    path: String,
    stream: S,
}

impl<S> Upload<S> {
    #[must_use]
    pub fn builder() -> UploadBuilder<S> {
        UploadBuilder::default()
    }
}

#[derive(Builder, Clone)]
pub struct UploadService {
    db_conn: DatabaseConnection,
    storage_backend: Arc<dyn StorageBackend>,
}

impl UploadService {
    #[must_use]
    pub fn builder() -> UploadServiceBuilder {
        UploadServiceBuilder::default()
    }

    pub async fn upload<S>(&self, upload: Upload<S>) -> Result<media_attachments::Model>
    where
        S: Stream<Item = Result<Bytes, BoxError>> + Send + 'static,
    {
        self.storage_backend
            .put(&upload.path, upload.stream.boxed())
            .await
            .map_err(Error::Storage)?;

        todo!();
    }
}
