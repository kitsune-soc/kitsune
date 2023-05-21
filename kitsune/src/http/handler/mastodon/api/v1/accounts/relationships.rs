use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::MastodonMapper,
    state::Zustand,
};
use axum::{debug_handler, extract::State, Json};
use axum_extra::extract::Query;
use diesel::ExpressionMethods;
use kitsune_db::{schema::accounts, PgPool};
use kitsune_type::mastodon::relationship::Relationship;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Deserialize, IntoParams)]
pub struct RelationshipQuery {
    id: Vec<Uuid>,
}

#[debug_handler(state = Zustand)]
#[utoipa::path(
    get,
    path = "/api/v1/accounts/relationships",
    security(
        ("oauth_token" = [])
    ),
    params(RelationshipQuery),
    responses(
        (status = 200, description = "Relationship between you and the other accounts", body = Vec<Relationship>),
        (status = 400, description = "One of the account IDs you input isn't known"),
    ),
)]
pub async fn get(
    AuthExtractor(user_data): MastodonAuthExtractor,
    State(db_conn): State<PgPool>,
    State(mastodon_mapper): State<MastodonMapper>,
    Query(query): Query<RelationshipQuery>,
) -> Result<Json<Vec<Relationship>>> {
    let mut db_conn = db_conn.get().await?;

    let relationships = accounts::table
        .filter(accounts::id.eq_any(&query.id))
        .load_stream(&mut db_conn)
        .await?
        .and_then(|account| mastodon_mapper.map((&user_data.account, &account)))
        .try_collect()
        .await?;

    Ok(Json(relationships))
}
