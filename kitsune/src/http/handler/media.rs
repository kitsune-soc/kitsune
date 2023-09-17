use crate::{error::Result, state::Zustand};
use axum::{
    body::HttpBody,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Router,
};
use http::header::CONTENT_TYPE;
use hyper::Body;
use kitsune_core::service::attachment::AttachmentService;
use speedy_uuid::Uuid;

async fn get(
    State(attachment_service): State<AttachmentService>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let attachment = attachment_service.get_by_id(id).await?;
    let stream = attachment_service.stream_file(&attachment).await?;

    Ok((
        [(CONTENT_TYPE, attachment.content_type)],
        Body::wrap_stream(stream).boxed(),
    )
        .into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/:id", routing::get(get))
}
