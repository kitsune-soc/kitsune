use crate::{
    error::Result, mapping::MastodonMapper, service::attachment::AttachmentService, state::Zustand,
};
use axum::{
    extract::{Multipart, Path, State},
    routing, Json, Router,
};
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

async fn put(
    State(attachment_service): State<AttachmentService>,
    multipart: Multipart,
) -> Result<Json<MediaAttachment>> {
    todo!();
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/:id", routing::get(get).put(put))
}
