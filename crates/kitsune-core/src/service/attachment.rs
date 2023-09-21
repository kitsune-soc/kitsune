use super::url::UrlService;
use crate::{
    consts::{MAX_MEDIA_DESCRIPTION_LENGTH, USER_AGENT},
    error::{ApiError, Error, Result, UploadError},
};
use bytes::{Bytes, BytesMut};
use derive_builder::Builder;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures_util::{pin_mut, stream, Stream, StreamExt, TryStreamExt};
use garde::Validate;
use img_parts::{DynImage, ImageEXIF};
use kitsune_db::{
    model::media_attachment::{MediaAttachment, NewMediaAttachment, UpdateMediaAttachment},
    schema::media_attachments,
    PgPool,
};
use kitsune_http_client::Client;
use kitsune_storage::{BoxError, Storage, StorageBackend};
use scoped_futures::ScopedFutureExt;
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

const ALLOWED_FILETYPES: &[mime::Name<'_>] = &[mime::IMAGE, mime::VIDEO, mime::AUDIO];

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_allowed_filetype(value: &str, _ctx: &()) -> garde::Result {
    let content_type: mime::Mime = value
        .parse()
        .map_err(|err: mime::FromStrError| garde::Error::new(err.to_string()))?;

    if !ALLOWED_FILETYPES.contains(&content_type.type_()) {
        return Err(garde::Error::new("Invalid file type"));
    }

    Ok(())
}

fn is_image_type_with_supported_metadata(mime: &str) -> bool {
    matches!(mime, "image/jpeg" | "image/png" | "image/webp")
}

#[derive(TypedBuilder, Validate)]
pub struct Update {
    #[garde(skip)]
    account_id: Uuid,
    #[garde(skip)]
    attachment_id: Uuid,
    #[builder(setter(strip_option))]
    #[garde(length(max = MAX_MEDIA_DESCRIPTION_LENGTH))]
    description: Option<String>,
}

#[derive(Builder, Validate)]
#[builder(pattern = "owned")]
pub struct Upload<S> {
    #[garde(skip)]
    account_id: Uuid,
    #[garde(custom(is_allowed_filetype))]
    content_type: String,
    #[builder(default, setter(strip_option))]
    #[garde(length(max = MAX_MEDIA_DESCRIPTION_LENGTH))]
    description: Option<String>,
    #[builder(default = "Uuid::now_v7().to_string()")]
    #[garde(skip)]
    path: String,
    #[garde(skip)]
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
            .user_agent(USER_AGENT)
            .unwrap()
            .build()
    )]
    client: Client,
    db_pool: PgPool,
    media_proxy_enabled: bool,
    #[builder(setter(into))]
    storage_backend: Storage,
    url_service: UrlService,
}

impl AttachmentService {
    pub async fn get_by_id(&self, id: Uuid) -> Result<MediaAttachment> {
        self.db_pool
            .with_connection(|db_conn| {
                media_attachments::table
                    .find(id)
                    .get_result(db_conn)
                    .scoped()
            })
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
        update.validate(&())?;

        let mut changeset = UpdateMediaAttachment::default();
        if let Some(ref description) = update.description {
            changeset = UpdateMediaAttachment {
                description: Some(description),
            };
        }

        self.db_pool
            .with_connection(|db_conn| {
                diesel::update(
                    media_attachments::table.filter(
                        media_attachments::id
                            .eq(update.attachment_id)
                            .and(media_attachments::account_id.eq(update.account_id)),
                    ),
                )
                .set(changeset)
                .get_result(db_conn)
                .scoped()
            })
            .await
            .map_err(Error::from)
    }

    pub async fn upload<S>(&self, upload: Upload<S>) -> Result<MediaAttachment>
    where
        S: Stream<Item = Result<Bytes, BoxError>> + Send + 'static,
    {
        upload.validate(&())?;

        // remove exif info from image uploads
        let upload_stream = if is_image_type_with_supported_metadata(&upload.content_type) {
            let stream = upload.stream;
            pin_mut!(stream);

            let mut img_bytes = BytesMut::new();
            while let Some(chunk) = stream
                .next()
                .await
                .transpose()
                .map_err(UploadError::StreamError)?
            {
                img_bytes.extend_from_slice(&chunk);
            }

            let img_bytes = img_bytes.freeze();
            let final_bytes = DynImage::from_bytes(img_bytes)
                .map_err(UploadError::ImageProcessingError)?
                .ok_or(img_parts::Error::WrongSignature)
                .map(|mut image| {
                    image.set_exif(None);
                    image.encoder().bytes()
                })
                .map_err(UploadError::ImageProcessingError)?;

            self.storage_backend
                .put(&upload.path, stream::once(async { Ok(final_bytes) }))
        } else {
            self.storage_backend.put(&upload.path, upload.stream)
        };

        upload_stream.await.map_err(Error::Storage)?;

        let media_attachment = self
            .db_pool
            .with_connection(|db_conn| {
                diesel::insert_into(media_attachments::table)
                    .values(NewMediaAttachment {
                        id: Uuid::now_v7(),
                        account_id: upload.account_id,
                        content_type: upload.content_type.as_str(),
                        description: upload.description.as_deref(),
                        blurhash: None,
                        file_path: Some(upload.path.as_str()),
                        remote_url: None,
                    })
                    .get_result(db_conn)
                    .scoped()
            })
            .await?;

        Ok(media_attachment)
    }
}

#[cfg(test)]
mod test {
    use std::convert::Infallible;

    use bytes::{Bytes, BytesMut};
    use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
    use futures_util::{future, pin_mut, stream, StreamExt};
    use http::{Request, Response};
    use hyper::Body;
    use img_parts::{
        jpeg::{markers, JpegSegment},
        ImageEXIF,
    };
    use iso8601_timestamp::Timestamp;
    use kitsune_db::{
        model::{
            account::{ActorType, NewAccount},
            media_attachment::MediaAttachment,
        },
        schema::accounts,
    };
    use kitsune_http_client::Client;
    use kitsune_storage::fs::Storage;
    use kitsune_test::database_test;
    use scoped_futures::ScopedFutureExt;
    use speedy_uuid::Uuid;
    use tempfile::TempDir;
    use tower::service_fn;

    use crate::{
        error::Error,
        service::{
            attachment::{AttachmentService, Upload},
            url::UrlService,
        },
    };

    #[tokio::test]
    #[serial_test::serial]
    async fn upload_jpeg() {
        database_test(|db_pool| async move {
            let client = Client::builder().service(service_fn(handle));

            let account_id = db_pool
                .with_connection(|db_conn| {
                    async move { Ok::<_, eyre::Report>(prepare_db(db_conn).await) }.scoped()
                })
                .await
                .unwrap();

            let temp_dir = TempDir::new().unwrap();
            let storage = Storage::new(temp_dir.path().to_owned());
            let url_service = UrlService::builder()
                .domain("example.com")
                .scheme("http")
                .build();

            let attachment_service = AttachmentService::builder()
                .client(client)
                .db_pool(db_pool)
                .url_service(url_service)
                .storage_backend(storage)
                .media_proxy_enabled(false)
                .build();

            let base = hex_simd::decode_to_vec("ffd8ffe000104a46494600010101004800480000ffdb004300030202020202030202020303030304060404040404080606050609080a0a090809090a0c0f0c0a0b0e0b09090d110d0e0f101011100a0c12131210130f101010ffc9000b080001000101011100ffcc000600101005ffda0008010100003f00d2cf20ffd9").unwrap();
            let mut jpeg = img_parts::jpeg::Jpeg::from_bytes(Bytes::from(base)).unwrap();

            let comment_segment = JpegSegment::new_with_contents(
                markers::APP1,
                Bytes::from("Exif\0\0Some info to be stripped")
            );
            jpeg.segments_mut().insert(1, comment_segment);
            assert!(jpeg.exif().is_some());

            let upload = Upload::builder()
                .content_type(String::from("image/jpeg"))
                .path(String::from("test.jpeg"))
                .stream(stream::once(future::ok(jpeg.encoder().bytes())))
                .account_id(account_id).build().unwrap();
            attachment_service.upload(upload).await.unwrap();

            let attachment = MediaAttachment {
                id: Uuid::now_v7(),
                account_id,
                content_type: String::from("image/jpeg"),
                description: None,
                blurhash: None,
                file_path: Some(String::from("test.jpeg")),
                remote_url: None,
                created_at: Timestamp::now_utc(),
                updated_at: Timestamp::now_utc()
            };
            let download = attachment_service.stream_file(&attachment).await.unwrap();

            let mut img_bytes = BytesMut::new();
            pin_mut!(download);
            while let Some(chunk) = download.next().await.transpose().unwrap() {
                img_bytes.extend_from_slice(&chunk);
            }
            let img_bytes = img_bytes.freeze();

            let jpeg = img_parts::jpeg::Jpeg::from_bytes(img_bytes).unwrap();
            assert!(jpeg.exif().is_none());
        })
        .await;
    }

    async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
        Ok::<_, Infallible>(Response::new(Body::from("")))
    }

    async fn prepare_db(db_conn: &mut AsyncPgConnection) -> Uuid {
        // Create a local user `@alice`
        db_conn
            .transaction(|tx| {
                async move {
                    let account_id = Uuid::now_v7();
                    diesel::insert_into(accounts::table)
                        .values(NewAccount {
                            id: account_id,
                            display_name: None,
                            username: "alice",
                            locked: false,
                            note: None,
                            local: true,
                            domain: "example.com",
                            actor_type: ActorType::Person,
                            url: "https://example.com/users/alice",
                            featured_collection_url: None,
                            followers_url: None,
                            following_url: None,
                            inbox_url: None,
                            outbox_url: None,
                            shared_inbox_url: None,
                            public_key_id: "https://example.com/users/alice#main-key",
                            public_key: "",
                            created_at: None,
                        })
                        .execute(tx)
                        .await?;
                    Ok::<_, Error>(account_id)
                }
                .scope_boxed()
            })
            .await
            .unwrap()
    }
}
