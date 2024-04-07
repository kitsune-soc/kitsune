use crate::http::extractor::{AuthExtractor, MastodonAuthExtractor};
use axum::{
    debug_handler,
    extract::{Path, State},
};
use http::StatusCode;
use kitsune_error::Result;
use kitsune_service::notification::NotificationService;
use speedy_uuid::Uuid;

#[debug_handler(state = crate::state::Zustand)]
#[utoipa::path(
    post,
    path = "/api/v1/notifications/{id}/dismiss",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = StatusCode::OK, description = "Notification has been dismissed"),
    )
)]
pub async fn post(
    State(notification_service): State<NotificationService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    notification_service
        .dismiss(id, user_data.account.id)
        .await?;

    Ok(StatusCode::OK)
}
