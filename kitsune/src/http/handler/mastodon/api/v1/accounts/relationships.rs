use crate::http::extractor::{AuthExtractor, MastodonAuthExtractor};
use axum::{Json, debug_handler, extract::State};
use axum_extra::extract::Query;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::StreamExt;
use kitsune_db::{PgPool, model::account::Account, schema::accounts, with_connection};
use kitsune_error::Result;
use kitsune_mastodon::MastodonMapper;
use kitsune_type::mastodon::relationship::Relationship;
use serde::Deserialize;
use speedy_uuid::Uuid;

#[derive(Deserialize)]
pub struct RelationshipQuery {
    id: Vec<Uuid>,
}

#[debug_handler(state = crate::state::Zustand)]
pub async fn get(
    AuthExtractor(user_data): MastodonAuthExtractor,
    State(db_pool): State<PgPool>,
    State(mastodon_mapper): State<MastodonMapper>,
    Query(query): Query<RelationshipQuery>,
) -> Result<Json<Vec<Relationship>>> {
    let mut account_stream = with_connection!(db_pool, |db_conn| {
        accounts::table
            .filter(accounts::id.eq_any(&query.id))
            .select(Account::as_select())
            .load_stream::<Account>(db_conn)
            .await
    })?;

    let mut relationships = Vec::with_capacity(query.id.len());
    while let Some(account) = account_stream.next().await.transpose()? {
        relationships.push(mastodon_mapper.map((&user_data.account, &account)).await?);
    }

    Ok(Json(relationships))
}
