use crate::{
    http::{
        extractor::{AgnosticForm, AuthExtractor, MastodonAuthExtractor},
        util::buffer_multipart_to_tempfile,
    },
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Multipart, Path, State},
    Json,
};
use futures_util::TryFutureExt;
use kitsune_error::{kitsune_error, Error, ErrorType, Result};
use kitsune_mastodon::MastodonMapper;
use kitsune_service::attachment::{AttachmentService, Update, Upload};
use kitsune_type::mastodon::MediaAttachment;
use serde::Deserialize;
use speedy_uuid::Uuid;

#[derive(Deserialize)]
pub struct UpdateAttachment {
    description: String,
}

pub async fn get(
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
                    let stream = buffer_multipart_to_tempfile(&mut field).await?;

                    upload = upload.content_type(content_type).stream(stream);
                }
                _ => continue,
            }
        }
    }

    let upload = upload.build().map_err(|err| {
        kitsune_error!(
            type = ErrorType::BadRequest.with_body(err.to_string()),
            "not all fields were filled"
        )
    })?;

    let media_attachment = attachment_service.upload(upload).await?;
    Ok(Json(mastodon_mapper.map(media_attachment).await?))
}

#[debug_handler(state = Zustand)]
pub async fn put(
    State(attachment_service): State<AttachmentService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(attachment_id): Path<Uuid>,
    AgnosticForm(form): AgnosticForm<UpdateAttachment>,
) -> Result<Json<MediaAttachment>> {
    let update = Update::builder()
        .account_id(user_data.account.id)
        .attachment_id(attachment_id)
        .description(form.description)
        .build();

    attachment_service
        .update(update)
        .map_err(Error::from)
        .and_then(|model| mastodon_mapper.map(model).map_err(Error::from))
        .map_ok(Json)
        .await
}
