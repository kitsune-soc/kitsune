use axum::{debug_handler, extract::State};
use http::StatusCode;

use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    service::notification::NotificationService,
    state::Zustand,
};

#[debug_handler(state = Zustand)]
#[utoipa::path(
    post,
    path = "/api/v1/notifications/clear",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = StatusCode::OK, description = "All notifications have been dismissed"),
    )
)]
pub async fn post(
    State(notification_service): State<NotificationService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
) -> Result<StatusCode> {
    notification_service.clear_all(user_data.account.id).await?;

    Ok(StatusCode::OK)
}
