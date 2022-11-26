use crate::{db::entity::user, error::Result, mapping::IntoMastodon, state::State};
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use sea_orm::EntityTrait;
use uuid::Uuid;

mod verify_credentials;

async fn get_handler(Extension(state): Extension<State>, Path(id): Path<Uuid>) -> Result<Response> {
    let Some(user) = user::Entity::find_by_id(id).one(&state.db_conn).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(user.into_mastodon(&state).await?).into_response())
}

pub fn routes() -> Router {
    Router::new()
        .route("/:id", get(get_handler))
        .route("/verify_credentials", get(verify_credentials::get))
}
