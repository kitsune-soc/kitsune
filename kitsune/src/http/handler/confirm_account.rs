use crate::{error::Result, state::AppState};
use axum::{
    debug_handler,
    extract::{Path, State},
    routing, Router,
};
use kitsune_core::service::user::UserService;
use serde::Deserialize;

#[derive(Deserialize)]
struct GetPath {
    confirmation_token: String,
}

#[debug_handler(state = AppState)]
async fn get(
    State(user_service): State<UserService>,
    Path(path): Path<GetPath>,
) -> Result<&'static str> {
    user_service
        .mark_as_confirmed_by_token(path.confirmation_token.as_str())
        .await?;

    Ok("Account confirmed successfully! You can log in now")
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/:confirmation_token", routing::get(get))
}
