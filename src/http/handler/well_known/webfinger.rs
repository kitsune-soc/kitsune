use crate::{db::entity::user, error::Result, state::State};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WebfingerQuery {
    resource: String,
}

#[derive(Deserialize, Serialize)]
pub struct Link {
    pub rel: String,
    pub href: String,
}

#[derive(Deserialize, Serialize)]
pub struct Webfinger {
    pub subject: String,
    pub aliases: Vec<String>,
    pub links: Vec<Link>,
}

pub async fn get(
    Extension(state): Extension<State>,
    Query(query): Query<WebfingerQuery>,
) -> Result<Response> {
    let username_at_instance = query.resource.trim_start_matches("acct:");
    let Some((username, instance)) = username_at_instance.split_once('@') else {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    };

    if instance != state.config.domain {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let Some(user) = user::Entity::find()
        .filter(
            user::Column::Username.eq(username)
                .and(user::Column::Domain.is_null()),
        )
        .one(&state.db_conn)
        .await?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(Webfinger {
        subject: query.resource,
        aliases: vec![user.url.clone()],
        links: vec![Link {
            rel: "self".into(),
            href: user.url,
        }],
    })
    .into_response())
}
