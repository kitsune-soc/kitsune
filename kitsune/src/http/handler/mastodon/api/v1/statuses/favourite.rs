use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    job::{deliver::favourite::FavouriteDeliveryContext, Job},
    mapping::IntoMastodon,
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use http::StatusCode;
use kitsune_db::{
    custom::JobState,
    entity::{favourites, jobs},
};
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use uuid::Uuid;

#[debug_handler(state = Zustand)]
pub async fn post(
    State(state): State<Zustand>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let Some(post) = state
        .service
        .post
        .get_by_id(id, Some(user_data.account.id))
        .await?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let id = Uuid::now_v7();
    let url = format!("https://{}/favourites/{id}", state.config.domain);
    let favourite = favourites::Model {
        id,
        account_id: user_data.account.id,
        post_id: post.id,
        url,
        created_at: Utc::now().into(),
    }
    .into_active_model()
    .insert(&state.db_conn)
    .await?;

    let context = Job::DeliverFavourite(FavouriteDeliveryContext {
        favourite_id: favourite.id,
    });

    jobs::Model {
        id: Uuid::now_v7(),
        state: JobState::Queued,
        run_at: Utc::now().into(),
        context: serde_json::to_value(context).unwrap(),
        fail_count: 0,
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    }
    .into_active_model()
    .insert(&state.db_conn)
    .await?;

    Ok(Json(post.into_mastodon(&state).await?).into_response())
}
