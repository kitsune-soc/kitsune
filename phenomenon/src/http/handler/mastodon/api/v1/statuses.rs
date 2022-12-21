use crate::{
    db::model::post::{self, Visibility},
    error::Result,
    http::extractor::AuthExtactor,
    mapping::IntoMastodon,
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

#[debug_handler(state = Zustand)]
async fn get_handler(
    State(state): State<Zustand>,
    AuthExtactor(_account): AuthExtactor,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let Some(post) = post::Entity::find()
        .filter(post::Column::Id.eq(id))
        .filter(
            post::Column::Visibility
                .eq(Visibility::Public)
                .or(post::Column::Visibility.eq(Visibility::Unlisted))
        )
        .one(&state.db_conn)
        .await?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(post.into_mastodon(&state).await?).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/:id", get(get_handler))
}
