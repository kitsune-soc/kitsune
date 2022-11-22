use crate::{db::entity::oauth::application, error::Result, state::State};
use askama::Template;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension, Form,
};
use sea_orm::EntityTrait;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AuthorizeQuery {
    response_type: String,
    client_id: Uuid,
    redirect_uri: String,
    scope: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
pub struct AuthorizeForm {
    username: String,
    password: String,
}

#[derive(Template)]
#[template(path = "authorize.html")]
struct AuthorizePage {
    app_name: String,
    domain: String,
}

pub async fn get(
    Extension(state): Extension<State>,
    Query(query): Query<AuthorizeQuery>,
) -> Result<Response> {
    let Some(application) =
        application::Entity::find_by_id(query.client_id)
            .one(&state.db_conn)
            .await?
    else {
        return Ok((StatusCode::BAD_REQUEST, "Client not found").into_response());
    };

    let page = AuthorizePage {
        app_name: application.name,
        domain: state.config.domain,
    }
    .render()
    .unwrap();
    Ok(Html(page).into_response())
}

pub async fn post(
    Extension(state): Extension<State>,
    Query(query): Query<AuthorizeQuery>,
    Form(form): Form<AuthorizeForm>,
) {
}
