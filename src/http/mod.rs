use std::io;

use self::handler::{oauth, posts, users, well_known};
use crate::state::State;
use axum::{
    http::StatusCode,
    routing::{get, get_service},
    Extension, Router,
};
use tower_http::{services::ServeDir, trace::TraceLayer};

pub mod graphql;

mod extractor;
mod handler;

#[allow(clippy::unused_async)]
async fn handle_error(err: io::Error) -> StatusCode {
    error!(error = %err, "Static file handler failed");
    StatusCode::INTERNAL_SERVER_ERROR
}

#[instrument(skip(state))]
pub async fn run(state: State, port: u16) {
    let router = Router::new()
        .route("/@:username", get(users::get))
        .route(
            "/public/*",
            get_service(ServeDir::new("public")).handle_error(handle_error),
        )
        .nest("/oauth", oauth::routes())
        .nest("/posts", posts::routes())
        .nest("/users", users::routes())
        .nest("/.well-known", well_known::routes())
        .merge(graphql::routes(state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(state))
        .into_make_service();

    axum::Server::bind(&([0, 0, 0, 0], port).into())
        .serve(router)
        .await
        .unwrap();
}
