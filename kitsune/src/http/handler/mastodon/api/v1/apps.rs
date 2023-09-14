use crate::state::Zustand;
use crate::{
    error::Result,
    http::extractor::FormOrJson,
    oauth2::{CreateApp, OAuth2Service},
};
use axum::{extract::State, routing, Json, Router};
use kitsune_type::mastodon::App;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct AppForm {
    client_name: String,
    redirect_uris: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/apps",
    request_body = AppForm,
    responses(
        (status = 200, description = "Newly created application", body = App),
    ),
)]
async fn post(
    State(oauth2): State<OAuth2Service>,
    FormOrJson(form): FormOrJson<AppForm>,
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

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::post(post))
}
