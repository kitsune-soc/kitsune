use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::MastodonMapper,
    state::Zustand,
};
use axum::{debug_handler, extract::State, Json};
use axum_extra::extract::Query;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::StreamExt;
use kitsune_db::{model::account::Account, schema::accounts, PgPool};
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
    let mut account_stream = accounts::table
        .filter(accounts::id.eq_any(&query.id))
        .select(Account::as_select())
        .load_stream::<Account>(&mut db_conn)
        .await?;

    let mut relationships = Vec::with_capacity(query.id.len());
    while let Some(account) = account_stream.next().await.transpose()? {
        relationships.push(mastodon_mapper.map((&user_data.account, &account)).await?);
    }

    Ok(Json(relationships))
}
