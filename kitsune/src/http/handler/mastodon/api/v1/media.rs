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

async fn post(
    State(attachment_service): State<AttachmentService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    mut multipart: Multipart,
) -> Result<Json<MediaAttachment>> {
    let mut upload = Upload::builder().account_id(user_data.account.id);
    while let Some(field) = multipart.next_field().await? {
        if let Some(field_name) = field.name() {
            match field_name {
                "description" => {
                    upload = upload.description(field.text().await?);
                }
                "file" => {
                    let Some(content_type) = field.content_type() else {
                        continue;
                    };

                    upload = upload
                        .content_type(content_type.to_string())
                        .stream(field.map_err(Into::into));

                    break;
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
    Router::new().route("/:id", routing::get(get).post(post))
}
