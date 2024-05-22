use crate::{
    consts::default_limit,
    http::{
        extractor::{AuthExtractor, MastodonAuthExtractor},
        pagination::{LinkHeader, PaginatedJsonResponse},
    },
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{OriginalUri, Path, State},
    routing, Json, Router,
};
use axum_extra::extract::Query;
use futures_util::{TryFutureExt, TryStreamExt};
use kitsune_error::{kitsune_error, Error, ErrorType, Result};
use kitsune_mastodon::MastodonMapper;
use kitsune_service::notification::{GetNotifications, NotificationService};
use kitsune_type::mastodon::{notification::NotificationType, Notification};
use kitsune_url::UrlService;
use serde::Deserialize;
use speedy_uuid::Uuid;

pub mod clear;
pub mod dismiss;

#[derive(Deserialize)]
pub struct GetQuery {
    #[serde(default)]
    max_id: Option<Uuid>,
    #[serde(default)]
    since_id: Option<Uuid>,
    #[serde(default)]
    min_id: Option<Uuid>,
    #[serde(default)]
    account_id: Option<Uuid>,
    #[serde(default)]
    types: Vec<NotificationType>,
    #[serde(default)]
    exclude_types: Vec<NotificationType>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[debug_handler(state = Zustand)]
pub async fn get(
    State(notification_service): State<NotificationService>,
    State(mastodon_mapper): State<MastodonMapper>,
    State(url_service): State<UrlService>,
    OriginalUri(original_uri): OriginalUri,
    Query(query): Query<GetQuery>,
    AuthExtractor(user_data): MastodonAuthExtractor,
) -> Result<PaginatedJsonResponse<Notification>> {
    let get_notifications = GetNotifications::builder()
        .receiving_account_id(user_data.account.id)
        .limit(query.limit)
        .since_id(query.since_id)
        .min_id(query.min_id)
        .max_id(query.max_id)
        .included_types(query.types.iter().copied().map(Into::into).collect())
        .excluded_types(
            query
                .exclude_types
                .iter()
                .copied()
                .map(Into::into)
                .collect(),
        )
        .triggering_account_id(query.account_id)
        .build();

    let notifications: Vec<Notification> = notification_service
        .get_notifications(get_notifications)
        .await?
        .map_err(Error::from)
        .and_then(|notif| mastodon_mapper.map(notif).map_err(Error::from))
        .try_collect()
        .await?;

    let link_header = LinkHeader::new(
        &notifications,
        query.limit,
        &url_service.base_url(),
        original_uri.path(),
        |a| a.id,
    );

    Ok((link_header, Json(notifications)))
}

#[debug_handler(state = Zustand)]
pub async fn get_by_id(
    State(notification_service): State<NotificationService>,
    State(mastodon_mapper): State<MastodonMapper>,
    Path(id): Path<Uuid>,
    AuthExtractor(user_data): MastodonAuthExtractor,
) -> Result<Json<Notification>> {
    let notification = notification_service
        .get_notification_by_id(id, user_data.account.id)
        .await?
        .ok_or_else(|| kitsune_error!(type = ErrorType::NotFound, "notification not found"))?;

    Ok(Json(mastodon_mapper.map(notification).await?))
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/", routing::get(get))
        .route("/:id", routing::get(get_by_id))
        .route("/:id/dismiss", routing::post(dismiss::post))
        .route("/clear", routing::post(clear::post))
}
