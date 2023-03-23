use crate::{
    error::{ApiError, Result},
    http::extractor::{AuthExtractor, FormOrJson, MastodonAuthExtractor},
    mapping::MastodonMapper,
    service::attachment::{AttachmentService, Update, Upload},
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Multipart, Path, State},
    routing, Json, Router,
};
use futures_util::{TryFutureExt, TryStreamExt};
use kitsune_type::mastodon::MediaAttachment;
use serde::Deserialize;
use std::io::SeekFrom;
use tempfile::tempfile;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
};
use tokio_util::io::ReaderStream;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema)]
pub struct CreateAttachment {
    pub description: Option<String>,
    #[schema(value_type = String, format = Binary)]
    pub file: (),
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateAttachment {
    description: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/media/{id}",
    responses(
        (status = 200, description = "Media attachment", body = MediaAttachment),
        (status = 404, description = "Media attachment doesn't exist"),
    ),
)]
pub async fn get(
    State(attachment_service): State<AttachmentService>,
    State(mapper): State<MastodonMapper>,
    Path(id): Path<Uuid>,
) -> Result<Json<MediaAttachment>> {
    Ok(Json(
        mapper.map(attachment_service.get_by_id(id).await?).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/media",
    security(
        ("oauth_token" = [])
    ),
    request_body(content = MediaAttachmentBody, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "New media attachment", body = MediaAttachment),
    ),
)]
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

    let upload = upload.build().map_err(|_| ApiError::BadRequest)?;
    let media_attachment = attachment_service.upload(upload).await?;
    Ok(Json(mastodon_mapper.map(media_attachment).await?))
}

#[debug_handler(state = Zustand)]
#[utoipa::path(
    put,
    path = "/api/v1/media/{id}",
    security(
        ("oauth_token" = [])
    ),
    request_body = UpdateAttachment,
    responses(
        (status = 200, description = "Updated media attachment", body = MediaAttachment),
    ),
)]
pub async fn put(
    State(attachment_service): State<AttachmentService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(attachment_id): Path<Uuid>,
    FormOrJson(form): FormOrJson<UpdateAttachment>,
) -> Result<Json<MediaAttachment>> {
    let update = Update::builder()
        .account_id(user_data.account.id)
        .attachment_id(attachment_id)
        .description(form.description)
        .build()
        .unwrap();

    attachment_service
        .update(update)
        .and_then(|model| mastodon_mapper.map(model))
        .map_ok(Json)
        .await
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/", routing::post(post))
        .route("/:id", routing::get(get).put(put))
}
