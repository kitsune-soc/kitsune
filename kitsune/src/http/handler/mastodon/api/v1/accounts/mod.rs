use crate::{error::Result, mapping::MastodonMapper, state::Zustand};
use axum::{
    extract::{Path, State},
    routing, Json, Router,
};
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{model::account::Account, schema::accounts, PgPool};
use kitsune_type::mastodon;
use uuid::Uuid;

pub mod follow;
pub mod lookup;
pub mod relationships;
pub mod statuses;
pub mod unfollow;
pub mod update_credentials;
pub mod verify_credentials;

#[utoipa::path(
    get,
    path = "/api/v1/accounts/{id}",
    responses(
        (status = 200, description = "Account information", body = Account),
        (status = StatusCode::NOT_FOUND, description = "No account with that ID exists"),
    )
)]
async fn get(
    State(db_conn): State<PgPool>,
    State(mastodon_mapper): State<MastodonMapper>,
    Path(id): Path<Uuid>,
) -> Result<Json<mastodon::Account>> {
    let mut db_conn = db_conn.get().await?;
    let account = accounts::table
        .find(id)
        .select(Account::as_select())
        .get_result::<Account>(&mut db_conn)
        .await?;

    Ok(Json(mastodon_mapper.map(account).await?))
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:id", routing::get(get))
        .route("/:id/follow", routing::post(follow::post))
        .route("/:id/statuses", routing::get(statuses::get))
        .route("/:id/unfollow", routing::post(unfollow::post))
        .route("/lookup", routing::get(lookup::get))
        .route("/relationships", routing::get(relationships::get))
        .route(
            "/update_credentials",
            routing::patch(update_credentials::patch),
        )
        .route("/verify_credentials", routing::get(verify_credentials::get))
}
