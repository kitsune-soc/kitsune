use axum::{
    debug_handler,
    extract::{Path, State},
};
use http::StatusCode;
use speedy_uuid::Uuid;

use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    service::notification::NotificationService,
    state::Zustand,
};

#[debug_handler(state = Zustand)]
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
