use crate::state::Zustand;
use axum::{
    debug_handler,
    extract::{Path, State},
};
use kitsune_email::MailingService;
use kitsune_error::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetPath {
    confirmation_token: String,
}

#[debug_handler(state = Zustand)]
pub async fn get(
    State(mailing_service): State<MailingService>,
    Path(path): Path<GetPath>,
) -> Result<&'static str> {
    mailing_service
        .mark_as_confirmed_by_token(path.confirmation_token.as_str())
        .await?;

    Ok("Account confirmed successfully! You can log in now")
}
