use crate::{error::Result, state::Zustand};
use axum::{
    debug_handler,
    extract::{Path, State},
    routing, Router,
};
use kitsune_email::MailingService;
use serde::Deserialize;

#[derive(Deserialize)]
struct GetPath {
    confirmation_token: String,
}

#[debug_handler(state = Zustand)]
async fn get(
    State(mailing_service): State<MailingService>,
    Path(path): Path<GetPath>,
) -> Result<&'static str> {
    mailing_service
        .mark_as_confirmed_by_token(path.confirmation_token.as_str())
        .await?;

    Ok("Account confirmed successfully! You can log in now")
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/:confirmation_token", routing::get(get))
}
