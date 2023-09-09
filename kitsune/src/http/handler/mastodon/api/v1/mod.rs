use axum::Router;
use kitsune_core::state::Zustand;

pub mod accounts;
pub mod apps;
pub mod follow_requests;
pub mod instance;
pub mod media;
pub mod notifications;
pub mod statuses;
pub mod timelines;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .nest("/apps", apps::routes())
        .nest("/accounts", accounts::routes())
        .nest("/follow_requests", follow_requests::routes())
        .nest("/instance", instance::routes())
        .nest("/media", media::routes())
        .nest("/notifications", notifications::routes())
        .nest("/statuses", statuses::routes())
        .nest("/timelines", timelines::routes())
}
