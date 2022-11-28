use crate::{db::entity::user, error::Result};
use axum::{extract::State, Json};
use phenomenon_model::mastodon::{
    instance::{Stats, Urls},
    Instance,
};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};

pub async fn get(State(state): State<crate::State>) -> Result<Json<Instance>> {
    let user_count = user::Entity::find()
        .filter(user::Column::Domain.is_null())
        .count(&state.db_conn)
        .await?;
    let domain_count = user::Entity::find()
        .filter(user::Column::Domain.is_not_null())
        .select_only()
        .column(user::Column::Domain)
        .group_by(user::Column::Domain)
        .count(&state.db_conn)
        .await?;

    Ok(Json(Instance {
        uri: state.config.domain.clone(),
        title: state.config.domain,
        short_description: "https://www.youtube.com/watch?v=6lnnPnr_0SU".into(),
        description: String::new(),
        email: String::new(),
        version: env!("CARGO_PKG_VERSION").into(),
        urls: Urls {
            streaming_api: String::new(),
        },
        stats: Stats {
            user_count,
            domain_count,
            status_count: 0,
        },
    }))
}
