use super::url::UrlService;
use crate::error::{ApiError, Error, Result};
use bytes::Bytes;
use derive_builder::Builder;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures_util::{Stream, StreamExt, TryStreamExt};
use kitsune_db::{
    model::media_attachment::{MediaAttachment, NewMediaAttachment, UpdateMediaAttachment},
    schema::media_attachments,
    PgPool,
};
use kitsune_http_client::Client;
use kitsune_storage::{BoxError, Storage, StorageBackend};
use typed_builder::TypedBuilder;
use uuid::Uuid;

const ALLOWED_FILETYPES: &[mime::Name<'_>] = &[mime::IMAGE, mime::VIDEO, mime::AUDIO];

#[derive(TypedBuilder)]
pub struct Update {
    account_id: Uuid,
    attachment_id: Uuid,
    #[builder(setter(strip_option))]
    description: Option<String>,
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

#[derive(Clone, TypedBuilder)]
pub struct AttachmentService {
    #[builder(default =
        Client::builder()
            .content_length_limit(None)
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .unwrap()
            .build()
    )]
    client: Client,
    db_conn: PgPool,
    media_proxy_enabled: bool,
    #[builder(setter(into))]
    storage_backend: Storage,
    url_service: UrlService,
}

impl AttachmentService {
    pub async fn get_by_id(&self, id: Uuid) -> Result<MediaAttachment> {
        let mut db_conn = self.db_conn.get().await?;
        media_attachments::table
            .find(id)
            .get_result(&mut db_conn)
            .await
            .map_err(Error::from)
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
        media_attachment: &MediaAttachment,
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

    pub async fn update(&self, update: Update) -> Result<MediaAttachment> {
        let mut changeset = UpdateMediaAttachment::default();
        if let Some(ref description) = update.description {
            changeset = UpdateMediaAttachment {
                description: Some(description),
            };
        }

        let mut db_conn = self.db_conn.get().await?;
        diesel::update(
            media_attachments::table.filter(
                media_attachments::id
                    .eq(update.attachment_id)
                    .and(media_attachments::account_id.eq(update.account_id)),
            ),
        )
        .set(changeset)
        .get_result(&mut db_conn)
        .await
        .map_err(Error::from)
    }

    pub async fn upload<S>(&self, upload: Upload<S>) -> Result<MediaAttachment>
    where
        S: Stream<Item = Result<Bytes, BoxError>> + Send + 'static,
    {
        let content_type: mime::Mime = upload.content_type.parse()?;
        if !ALLOWED_FILETYPES.contains(&content_type.type_()) {
            return Err(ApiError::UnsupportedMediaType.into());
        }

        self.storage_backend
            .put(&upload.path, upload.stream)
            .await
            .map_err(Error::Storage)?;

        let mut db_conn = self.db_conn.get().await?;
        let media_attachment = diesel::insert_into(media_attachments::table)
            .values(NewMediaAttachment {
                id: Uuid::now_v7(),
                account_id: upload.account_id,
                content_type: upload.content_type.as_str(),
                description: upload.description.as_deref(),
                blurhash: None,
                file_path: Some(upload.path.as_str()),
                remote_url: None,
            })
            .get_result(&mut db_conn)
            .await?;

        Ok(media_attachment)
    }
}
