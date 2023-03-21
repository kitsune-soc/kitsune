use super::url::UrlService;
use crate::error::{ApiError, Error, Result};
use bytes::Bytes;
use chrono::Utc;
use derive_builder::Builder;
use futures_util::{Stream, StreamExt, TryStreamExt};
use kitsune_db::entity::{media_attachments, prelude::MediaAttachments};
use kitsune_http_client::Client;
use kitsune_storage::{BoxError, StorageBackend};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter,
};
use std::sync::Arc;
use uuid::Uuid;

const ALLOWED_FILETYPES: &[mime::Name<'_>] = &[mime::IMAGE, mime::VIDEO, mime::AUDIO];

#[derive(Builder)]
pub struct Update {
    account_id: Uuid,
    attachment_id: Uuid,
    #[builder(setter(strip_option))]
    description: Option<String>,
}

impl Update {
    #[must_use]
    pub fn builder() -> UpdateBuilder {
        UpdateBuilder::default()
    }
}

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct Upload<S> {
    account_id: Uuid,
    content_type: String,
    #[builder(default, setter(strip_option))]
    description: Option<String>,
    #[builder(default = "Uuid::now_v7().to_string()")]
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
pub struct AttachmentService {
    #[builder(default = "
        Client::builder()
            .content_length_limit(None)
            .user_agent(concat!(
                env!(\"CARGO_PKG_NAME\"),
                \"/\",
                env!(\"CARGO_PKG_VERSION\")
            ))
            .unwrap()
            .build()
    ")]
    client: Client,
    db_conn: DatabaseConnection,
    media_proxy_enabled: bool,
    storage_backend: Arc<dyn StorageBackend>,
    url_service: UrlService,
}

impl AttachmentService {
    #[must_use]
    pub fn builder() -> AttachmentServiceBuilder {
        AttachmentServiceBuilder::default()
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<media_attachments::Model> {
        MediaAttachments::find_by_id(id)
            .one(&self.db_conn)
            .await?
            .ok_or_else(|| ApiError::NotFound.into())
    }

    /// Get the URL to an attachment
    ///
    /// Automatically adjusts the URL to whether the media proxy is enabled
    ///
    /// # Panics
    ///
    /// This should never panic
    pub async fn get_url(&self, id: Uuid) -> Result<String> {
        let media_attachment = self.get_by_id(id).await?;
        if self.media_proxy_enabled || media_attachment.file_path.is_some() {
            return Ok(self.url_service.media_url(id));
        }

        Ok(media_attachment.remote_url.unwrap())
    }

    /// Return a stream that yields the file's contents
    ///
    /// # Panics
    ///
    /// This should never panic
    pub async fn stream_file(
        &self,
        media_attachment: &media_attachments::Model,
    ) -> Result<impl Stream<Item = Result<Bytes>>> {
        if let Some(ref file_path) = media_attachment.file_path {
            let stream = self
                .storage_backend
                .get(file_path.as_str())
                .await
                .map_err(Error::Storage)?;

            Ok(stream.map_err(Error::Storage).left_stream())
        } else if self.media_proxy_enabled {
            Ok(self
                .client
                .get(media_attachment.remote_url.as_ref().unwrap())
                .await?
                .stream()
                .map_err(Into::into)
                .right_stream())
        } else {
            Err(ApiError::NotFound.into())
        }
    }

    pub async fn update(&self, update: Update) -> Result<media_attachments::Model> {
        let active_model = media_attachments::ActiveModel {
            id: ActiveValue::Set(update.attachment_id),
            description: update
                .description
                .map_or(ActiveValue::NotSet, |desc| ActiveValue::Set(Some(desc))),
            ..Default::default()
        };

        MediaAttachments::update(active_model)
            .filter(media_attachments::Column::AccountId.eq(update.account_id))
            .exec(&self.db_conn)
            .await
            .map_err(Error::from)
    }

    pub async fn upload<S>(&self, upload: Upload<S>) -> Result<media_attachments::Model>
    where
        S: Stream<Item = Result<Bytes, BoxError>> + Send + 'static,
    {
        let content_type: mime::Mime = upload.content_type.parse()?;
        if !ALLOWED_FILETYPES.contains(&content_type.type_()) {
            return Err(ApiError::UnsupportedMediaType.into());
        }

        self.storage_backend
            .put(&upload.path, upload.stream.boxed())
            .await
            .map_err(Error::Storage)?;

        let media_attachment = media_attachments::Model {
            id: Uuid::now_v7(),
            account_id: upload.account_id,
            content_type: upload.content_type,
            description: upload.description,
            blurhash: None,
            file_path: Some(upload.path),
            remote_url: None,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }
        .into_active_model()
        .insert(&self.db_conn)
        .await?;

        Ok(media_attachment)
    }
}
