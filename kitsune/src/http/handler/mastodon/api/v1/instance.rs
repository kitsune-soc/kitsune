use crate::{error::Result, service::url::UrlService, state::Zustand};
use axum::{extract::State, routing, Json, Router};
use kitsune_db::entity::{
    accounts,
    prelude::{Accounts, Users},
};
use kitsune_type::mastodon::{
    instance::{Stats, Urls},
    Instance,
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
};

async fn get(
    State(db_conn): State<DatabaseConnection>,
    State(url_service): State<UrlService>,
) -> Result<Json<Instance>> {
    let user_count = Users::find().count(&db_conn).await?;

    let domain_count = Accounts::find()
        .filter(accounts::Column::Domain.is_not_null())
        .select_only()
        .column(accounts::Column::Domain)
        .group_by(accounts::Column::Domain)
        .count(&db_conn)
        .await?;

    Ok(Json(Instance {
        uri: url_service.domain().into(),
        title: "Kitsune".into(),
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

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
