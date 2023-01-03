use crate::{
    db::model::post::{self, Visibility},
    error::Result,
    mapping::IntoActivity,
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

#[debug_handler]
pub async fn get(State(state): State<Zustand>, Path(id): Path<Uuid>) -> Result<Response> {
    let Some(post) = post::Entity::find_by_id(id)
        .filter(post::Column::Visibility.is_in([Visibility::Public, Visibility::Unlisted]))
        .one(&state.db_conn)
        .await?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(post.into_activity(&state).await?).into_response())
}
