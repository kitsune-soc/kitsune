use crate::{http::extractor::SignedActivity, state::State};
use axum::{debug_handler, Extension};

#[debug_handler]
pub async fn post(Extension(_state): Extension<State>, SignedActivity(activity): SignedActivity) {
    // TODO: Insert activity into database

    match activity.rest.r#type.as_str() {
        "Follow" => (),
        "Undo" => (),
        "Delete" => (),
        "Announce" => (),
        "Create" => (),
        "Like" => (),
        "Update" => (),
        _ => (),
    }
}
