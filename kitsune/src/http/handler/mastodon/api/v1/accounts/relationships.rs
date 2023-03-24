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
