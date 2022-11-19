use self::handler::{posts, users, well_known};
use crate::state::State;
use axum::{routing::get, Extension, Router};
use tower_http::trace::TraceLayer;

pub mod graphql;

mod extractor;
mod handler;

#[instrument(skip(state))]
pub async fn run(state: State, port: u16) {
    let router = Router::new()
        .route("/@:username", get(users::get))
        .nest("/posts", posts::routes())
        .nest("/users", users::routes())
        .nest("/.well-known", well_known::routes())
        .merge(graphql::routes(state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(state))
        .into_make_service();

    axum::Server::bind(&([127, 0, 0, 1], port).into())
        .serve(router)
        .await
        .unwrap();
}
