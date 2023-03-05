use crate::{
    error::{ApiError, Result},
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::MastodonMapper,
    service::attachment::{AttachmentService, Upload},
    state::Zustand,
};
use axum::{
    extract::{Multipart, Path, State},
    routing, Json, Router,
};
use futures_util::TryStreamExt;
use kitsune_type::mastodon::MediaAttachment;
use std::io::SeekFrom;
use tempfile::tempfile;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

async fn get(
    State(attachment_service): State<AttachmentService>,
    State(mapper): State<MastodonMapper>,
    Path(id): Path<Uuid>,
) -> Result<Json<MediaAttachment>> {
    Ok(Json(
        mapper.map(attachment_service.get_by_id(id).await?).await?,
    ))
}

pub async fn post(
    State(attachment_service): State<AttachmentService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    mut multipart: Multipart,
) -> Result<Json<MediaAttachment>> {
    let mut upload = Upload::builder().account_id(user_data.account.id);
    while let Some(mut field) = multipart.next_field().await? {
        if let Some(field_name) = field.name() {
            match field_name {
                "description" => {
                    upload = upload.description(field.text().await?);
                }
                "file" => {
                    let Some(content_type) = field.content_type().map(ToString::to_string) else {
                        continue;
                    };

                    let tempfile = tempfile().unwrap();
                    let mut tempfile = File::from_std(tempfile);

                    while let Some(chunk) = field.chunk().await? {
                        if let Err(err) = tempfile.write_all(&chunk).await {
                            error!(error = ?err, "Failed to write chunk to tempfile");
                            return Err(ApiError::InternalServerError.into());
                        }
                    }

                    tempfile.seek(SeekFrom::Start(0)).await.unwrap();

                    upload = upload
                        .content_type(content_type)
                        .stream(ReaderStream::new(tempfile).map_err(Into::into));
                }
                _ => continue,
            }
        }
    }

    let upload = upload.build().map_err(|_| ApiError::Unauthorised)?;
    let media_attachment = attachment_service.upload(upload).await?;
    Ok(Json(mastodon_mapper.map(media_attachment).await?))
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/", routing::post(post))
        .route("/:id", routing::get(get))
}
