use crate::state::Zustand;
use axum::Router;

mod accounts;
mod apps;
mod instance;
mod statuses;

pub fn routes() -> Router<Zustand> {
    Router::new()
        .nest("/apps", apps::routes())
        .nest("/instance", instance::routes())
        .nest("/accounts", accounts::routes())
        .nest("/statuses", statuses::routes())
}
