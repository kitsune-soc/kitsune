use crate::{db::model::account, error::Result, mapping::IntoObject, state::Zustand};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{self, post},
    Json, Router,
};
use http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

mod followers;
mod following;
mod inbox;
mod outbox;

async fn get(State(state): State<Zustand>, Path(username): Path<String>) -> Result<Response> {
    let Some(account) = account::Entity::find()
        .filter(account::Column::Username.eq(username).and(account::Column::Domain.is_null()))
        .one(&state.db_conn)
        .await? else {
            return Ok(StatusCode::NOT_FOUND.into_response());
        };

    Ok(Json(account.into_object(&state).await?).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:username", routing::get(get))
        .route("/:username/followers", routing::get(followers::get))
        .route("/:username/following", routing::get(following::get))
        .route("/:username/inbox", post(inbox::post))
        .route("/:username/outbox", routing::get(outbox::get))
}
