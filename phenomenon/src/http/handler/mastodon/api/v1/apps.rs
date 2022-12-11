use crate::{
    db::model::oauth::application, error::Result, http::extractor::FormOrJson,
    util::generate_secret,
};
use axum::{extract::State, Json};
use chrono::Utc;
use phenomenon_model::mastodon::App;
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AppForm {
    client_name: String,
    redirect_uris: String,
}

pub async fn post(
    State(db_conn): State<DatabaseConnection>,
    FormOrJson(form): FormOrJson<AppForm>,
) -> Result<Json<App>> {
    let application = application::Model {
        id: Uuid::new_v4(),
        name: form.client_name,
        secret: generate_secret(),
        redirect_uri: form.redirect_uris,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
    .into_active_model()
    .insert(&db_conn)
    .await?;

    Ok(Json(App {
        id: application.id,
        name: application.name,
        redirect_uri: application.redirect_uri,
        client_id: application.id,
        client_secret: application.secret,
    }))
}
