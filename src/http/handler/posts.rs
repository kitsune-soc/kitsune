use crate::{db::entity::post, error::Result, mapping::IntoActivityPub, state::State};
use axum::{
    debug_handler,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Extension, Json, Router,
};
use phenomenon_model::ap::Privacy;
use sea_orm::EntityTrait;
use uuid::Uuid;

#[debug_handler]
async fn get(Extension(state): Extension<State>, Path(id): Path<Uuid>) -> Result<Response> {
    let Some(post) = post::Entity::find_by_id(id).one(&state.db_conn).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let note = post.into_activitypub(&state).await?;
    // Just act like the note doesn't exist if it's private
    // Obviously there is timing attack potential here but, honestly, WHO CARES?
    if note.is_private() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    Ok(Json(note).into_response())
}

pub fn routes() -> Router {
    Router::new().route("/:id", routing::get(get))
}
