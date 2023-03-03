use derive_builder::Builder;
use kitsune_storage::StorageBackend;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

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
}
