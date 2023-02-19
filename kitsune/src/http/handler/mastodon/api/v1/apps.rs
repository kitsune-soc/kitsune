use crate::{
    error::Result, http::extractor::FormOrJson, service::oauth2::CreateApp, state::Zustand,
};
use axum::{extract::State, routing, Json, Router};
use kitsune_type::mastodon::App;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppForm {
    client_name: String,
    redirect_uris: String,
}

async fn post(
    State(state): State<Zustand>,
    FormOrJson(form): FormOrJson<AppForm>,
) -> Result<Json<App>> {
    let create_app = CreateApp::builder()
        .name(form.client_name)
        .redirect_uris(form.redirect_uris)
        .build()
        .unwrap();
    let application = state.service.oauth2.create_app(create_app).await?;

    Ok(Json(App {
        id: application.id,
        name: application.name,
        redirect_uri: application.redirect_uri,
        client_id: application.id,
        client_secret: application.secret,
    }))
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::post(post))
}
