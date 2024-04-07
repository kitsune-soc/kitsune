use crate::state::Zustand;
use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Router,
};
use http::header::CONTENT_TYPE;
use kitsune_error::Result;
use kitsune_service::attachment::AttachmentService;
use speedy_uuid::Uuid;

async fn get(
    State(attachment_service): State<AttachmentService>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let attachment = attachment_service.get_by_id(id).await?;
    let stream = attachment_service.stream_file(&attachment).await?;

    Ok((
        [(CONTENT_TYPE, attachment.content_type)],
        Body::from_stream(stream),
    )
        .into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/:id", routing::get(get))
}
