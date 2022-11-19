use crate::{db::entity::user, error::Result, mapping::IntoActivityPub, state::State};
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{self, post},
    Extension, Json, Router,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub mod inbox;

pub async fn get(
    Extension(state): Extension<State>,
    Path(username): Path<String>,
) -> Result<Response> {
    let Some(user) = user::Entity::find()
        .filter(user::Column::Username.eq(username))
        .filter(user::Column::Domain.is_null())
        .one(&state.db_conn)
        .await? else {
            return Ok(StatusCode::NOT_FOUND.into_response());
        };

    Ok(Json(user.into_activitypub(&state).await?).into_response())
}

pub fn routes() -> Router {
    Router::new()
        .route("/:username", routing::get(get))
        .route("/:username/inbox", post(inbox::post))
}
