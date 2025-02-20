use crate::{
    http::extractor::AgnosticForm,
    oauth2::{CreateApp, OAuth2Service},
};
use axum::{Json, extract::State};
use kitsune_error::Result;
use kitsune_type::mastodon::App;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppForm {
    client_name: String,
    redirect_uris: String,
}

pub async fn post(
    State(oauth2): State<OAuth2Service>,
    AgnosticForm(form): AgnosticForm<AppForm>,
) -> Result<Json<App>> {
    let create_app = CreateApp::builder()
        .name(form.client_name)
        .redirect_uris(form.redirect_uris)
        .build();
    let application = oauth2.create_app(create_app).await?;

    Ok(Json(App {
        id: application.id,
        name: application.name,
        redirect_uri: application.redirect_uri,
        client_id: application.id,
        client_secret: application.secret,
    }))
}
