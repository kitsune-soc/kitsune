use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
};
use axum::{debug_handler, extract::State, Json};
use axum_extra::extract::Query;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::StreamExt;
use kitsune_core::mapping::MastodonMapper;
use kitsune_db::{model::account::Account, schema::accounts, PgPool};
use kitsune_type::mastodon::relationship::Relationship;
use scoped_futures::ScopedFutureExt;
use serde::Deserialize;
use speedy_uuid::Uuid;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct RelationshipQuery {
    id: Vec<Uuid>,
}

#[debug_handler(state = crate::state::AppState)]
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
    State(db_pool): State<PgPool>,
    State(mastodon_mapper): State<MastodonMapper>,
    Query(query): Query<RelationshipQuery>,
) -> Result<Json<Vec<Relationship>>> {
    let mut account_stream = db_pool
        .with_connection(|db_conn| {
            accounts::table
                .filter(accounts::id.eq_any(&query.id))
                .select(Account::as_select())
                .load_stream::<Account>(db_conn)
                .scoped()
        })
        .await?;

    let mut relationships = Vec::with_capacity(query.id.len());
    while let Some(account) = account_stream.next().await.transpose()? {
        relationships.push(mastodon_mapper.map((&user_data.account, &account)).await?);
    }

    Ok(Json(relationships))
}
