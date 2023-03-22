use crate::{
    error::{ApiError, Result},
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::MastodonMapper,
    state::Zustand,
};
use axum::{debug_handler, extract::State, Json};
use axum_extra::extract::Query;
use kitsune_db::entity::prelude::Accounts;
use kitsune_type::mastodon::relationship::Relationship;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RelationshipQuery {
    id: Vec<Uuid>,
}

#[debug_handler(state = Zustand)]
pub async fn get(
    AuthExtractor(user_data): MastodonAuthExtractor,
    State(db_conn): State<DatabaseConnection>,
    State(mastodon_mapper): State<MastodonMapper>,
    Query(query): Query<RelationshipQuery>,
) -> Result<Json<Vec<Relationship>>> {
    let mut relationships = Vec::with_capacity(query.id.len());
    for account_id in query.id {
        let Some(account) = Accounts::find_by_id(account_id).one(&db_conn).await? else {
            return Err(ApiError::BadRequest.into());
        };

        relationships.push(mastodon_mapper.map((&user_data.account, &account)).await?);
    }

    Ok(Json(relationships))
}
