use crate::state::Zustand;
use axum::Router;

mod accounts;
mod apps;
mod instance;
mod statuses;
mod timelines;

pub mod media;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .nest("/apps", apps::routes())
        .nest("/accounts", accounts::routes())
        .nest("/instance", instance::routes())
        .nest("/media", media::routes())
        .nest("/statuses", statuses::routes())
        .nest("/timelines", timelines::routes())
}
