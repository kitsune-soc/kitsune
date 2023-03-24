use crate::state::Zustand;
use axum::Router;

pub mod accounts;
pub mod apps;
pub mod instance;
pub mod media;
pub mod statuses;
pub mod timelines;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .nest("/apps", apps::routes())
        .nest("/accounts", accounts::routes())
        .nest("/instance", instance::routes())
        .nest("/media", media::routes())
        .nest("/statuses", statuses::routes())
        .nest("/timelines", timelines::routes())
}
