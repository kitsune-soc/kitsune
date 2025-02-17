use bytes::Bytes;
use derive_builder::Builder;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures_util::{stream, Stream, StreamExt, TryStreamExt};
use garde::Validate;
use img_parts::{DynImage, ImageEXIF};
use kitsune_core::consts::{MAX_MEDIA_DESCRIPTION_LENGTH, USER_AGENT};
use kitsune_db::{
    model::media_attachment::{MediaAttachment, NewMediaAttachment, UpdateMediaAttachment},
    schema::media_attachments,
    with_connection, PgPool,
};
use kitsune_derive::kitsune_service;
use kitsune_error::{kitsune_error, Error, ErrorType, Result};
use kitsune_http_client::Client;
use kitsune_storage::{AnyStorageBackend, StorageBackend};
use kitsune_url::UrlService;
use memmap2::Mmap;
use speedy_uuid::Uuid;
use std::pin::pin;
use tempfile::tempfile;
use tokio::{fs::File, io::AsyncWriteExt};
use typed_builder::TypedBuilder;

const ALLOWED_FILETYPES: &[mime::Name<'_>] = &[mime::IMAGE, mime::VIDEO, mime::AUDIO];

fn is_allowed_filetype<T>(value: &str, _ctx: &T) -> garde::Result {
    let content_type: mime::Mime = value
        .parse()
        .map_err(|err: mime::FromStrError| garde::Error::new(err.to_string()))?;

    if !ALLOWED_FILETYPES.contains(&content_type.type_()) {
        return Err(garde::Error::new("Invalid file type"));
    }

    Ok(())
}

fn is_image_type_with_supported_metadata(mime: &str) -> bool {
    // TODO: migrate the match to use the mime crate enums
    matches!(mime, "image/jpeg" | "image/png" | "image/webp")
}

#[derive(TypedBuilder, Validate)]
pub struct Update {
    #[garde(skip)]
    account_id: Uuid,
    #[garde(skip)]
    attachment_id: Uuid,
    #[builder(setter(strip_option))]
    #[garde(length(chars, max = MAX_MEDIA_DESCRIPTION_LENGTH))]
    description: Option<String>,
}

#[derive(Builder, Validate)]
#[builder(pattern = "owned")]
pub struct Upload<S> {
    #[builder(default, setter(strip_option))]
    #[garde(skip)]
    account_id: Option<Uuid>,
    #[garde(custom(is_allowed_filetype))]
    content_type: String,
    #[builder(default, setter(strip_option))]
    #[garde(length(chars, max = MAX_MEDIA_DESCRIPTION_LENGTH))]
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

#[kitsune_service]
pub struct AttachmentService {
    #[builder(default =
        Client::builder()
            .content_length_limit(None)
            .user_agent(USER_AGENT)
            .unwrap()
            .build()
    )]
    http_client: Client,
    db_pool: PgPool,
    media_proxy_enabled: bool,
    #[builder(setter(into))]
    storage_backend: AnyStorageBackend,
    url_service: UrlService,
}

impl AttachmentService {
    pub async fn get_by_id(&self, id: Uuid) -> Result<MediaAttachment> {
        with_connection!(self.db_pool, |db_conn| {
            media_attachments::table.find(id).get_result(db_conn).await
        })
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
            return Ok(self.url_service.media_url(media_attachment.id));
        }

        Ok(media_attachment.remote_url.as_ref().unwrap().to_string())
    }

    /// Return a stream that yields the file's contents
    ///
    /// # Panics
    ///
    /// This should never panic
    pub async fn stream_file(
        &self,
        media_attachment: &MediaAttachment,
    ) -> Result<impl Stream<Item = Result<Bytes>> + 'static> {
        // TODO: Find way to avoid boxing the streams here
        if let Some(ref file_path) = media_attachment.file_path {
            let stream = self.storage_backend.get(file_path.as_str()).await?;

            Ok(stream.map_err(Error::from).boxed())
        } else if self.media_proxy_enabled {
            Ok(self
                .http_client
                .get(media_attachment.remote_url.as_ref().unwrap())
                .await?
                .stream()
                .map_err(Into::into)
                .boxed())
        } else {
            Err(kitsune_error!(type = ErrorType::NotFound, "attachment not found"))
        }
    }

    pub async fn update(&self, update: Update) -> Result<MediaAttachment> {
        update.validate()?;

        let mut changeset = UpdateMediaAttachment::default();
        if let Some(ref description) = update.description {
            changeset = UpdateMediaAttachment {
                description: Some(description),
            };
        }

        with_connection!(self.db_pool, |db_conn| {
            diesel::update(
                media_attachments::table.filter(
                    media_attachments::id
                        .eq(update.attachment_id)
                        .and(media_attachments::account_id.eq(update.account_id)),
                ),
            )
            .set(changeset)
            .get_result(db_conn)
            .await
        })
        .map_err(Error::from)
    }

    pub async fn upload<S>(&self, upload: Upload<S>) -> Result<MediaAttachment>
    where
        S: Stream<Item = Result<Bytes>> + Send + Sync + 'static,
    {
        upload.validate()?;

        // remove exif info from image uploads
        if is_image_type_with_supported_metadata(&upload.content_type) {
            let mut stream = pin!(upload.stream);

            let mut tempfile = File::from_std(tempfile()?);
            while let Some(chunk) = stream.try_next().await? {
                tempfile.write_all(&chunk).await?;
            }
            tempfile.flush().await?;

            // SAFETY: Idk man. We vibe, we vibe.
            #[allow(unsafe_code)]
            let tempfile_mmap = unsafe { Mmap::map(&tempfile)? };
            let img_bytes = Bytes::from_owner(tempfile_mmap);

            let encoder = DynImage::from_bytes(img_bytes)?
                .ok_or(img_parts::Error::WrongSignature)
                .map(|mut image| {
                    image.set_exif(None);
                    image.encoder()
                })?;

            let clean_stream = stream::iter(encoder.map(Ok::<_, kitsune_error::Error>));
            self.storage_backend.put(&upload.path, clean_stream).await?;
        } else {
            self.storage_backend
                .put(&upload.path, upload.stream)
                .await?;
        }

        let media_attachment = with_connection!(self.db_pool, |db_conn| {
            diesel::insert_into(media_attachments::table)
                .values(NewMediaAttachment {
                    id: Uuid::now_v7(),
                    content_type: upload.content_type.as_str(),
                    account_id: upload.account_id,
                    description: upload.description.as_deref(),
                    blurhash: None,
                    file_path: Some(upload.path.as_str()),
                    remote_url: None,
                })
                .get_result(db_conn)
                .await
        })?;

        Ok(media_attachment)
    }
}

#[cfg(test)]
mod test {
    use crate::attachment::{AttachmentService, Upload};
    use bytes::{Bytes, BytesMut};
    use diesel_async::{AsyncPgConnection, RunQueryDsl};
    use futures_util::{future, stream, StreamExt};
    use http::{Request, Response};
    use http_body_util::Empty;
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
        with_connection_panicky,
    };
    use kitsune_http_client::Client;
    use kitsune_storage::fs::Storage;
    use kitsune_test::database_test;
    use kitsune_url::UrlService;
    use speedy_uuid::Uuid;
    use std::{convert::Infallible, pin::pin};
    use tempfile::TempDir;
    use tower::service_fn;

    #[tokio::test]
    async fn upload_jpeg() {
        database_test(|db_pool| async move {
            let client = Client::builder().service(service_fn(handle));

            let account_id = with_connection_panicky!(db_pool, |db_conn| {
                Ok::<_, eyre::Report>(prepare_db(db_conn).await)
            })
            .unwrap();

            let temp_dir = TempDir::new().unwrap();
            let storage = Storage::new(temp_dir.path().to_owned());
            let url_service = UrlService::builder()
                .domain("example.com")
                .scheme("http")
                .build();

            let attachment_service = AttachmentService::builder()
                .http_client(client)
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
                account_id: Some(account_id),
                content_type: String::from("image/jpeg"),
                description: None,
                blurhash: None,
                file_path: Some(String::from("test.jpeg")),
                remote_url: None,
                created_at: Timestamp::now_utc(),
                updated_at: Timestamp::now_utc()
            };
            let mut download = pin!(attachment_service.stream_file(&attachment).await.unwrap());

            let mut img_bytes = BytesMut::new();
            while let Some(chunk) = download.next().await.transpose().unwrap() {
                img_bytes.extend_from_slice(&chunk);
            }
            let img_bytes = img_bytes.freeze();

            let jpeg = img_parts::jpeg::Jpeg::from_bytes(img_bytes).unwrap();
            assert!(jpeg.exif().is_none());
        })
        .await;
    }

    async fn handle(
        _req: Request<kitsune_http_client::Body>,
    ) -> Result<Response<Empty<Bytes>>, Infallible> {
        Ok::<_, Infallible>(Response::new(Empty::new()))
    }

    async fn prepare_db(db_conn: &mut AsyncPgConnection) -> Uuid {
        // Create a local user `@alice`
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
            .execute(db_conn)
            .await
            .unwrap();

        account_id
    }
}
