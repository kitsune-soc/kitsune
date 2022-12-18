use crate::{db::model::account, error::Result, mapping::IntoActivityPub, state::Zustand};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{self, post},
    Json, Router,
};
use http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub mod inbox;

async fn get(State(state): State<Zustand>, Path(username): Path<String>) -> Result<Response> {
    let Some(account) = account::Entity::find()
        .filter(account::Column::Username.eq(username).and(account::Column::Domain.is_null()))
        .one(&state.db_conn)
        .await? else {
            return Ok(StatusCode::NOT_FOUND.into_response());
        };

    Ok(Json(account.into_activitypub(&state).await?).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:username", routing::get(get))
        .route("/:username/inbox", post(inbox::post))
}
