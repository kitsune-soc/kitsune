use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

pub struct ActivityPubJson<T>(pub T);

impl<T> IntoResponse for ActivityPubJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        (
            [("Content-Type", "application/activity+json")],
            Json(self.0),
        )
            .into_response()
    }
}
