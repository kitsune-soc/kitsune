use askama::Template;
use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
struct AuthorizeQuery {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    scope: Option<String>,
    state: Option<String>,
}

#[derive(Template)]
#[template(path = "authorize.html")]
struct AuthorizePage {
    
}

pub async fn get(Query(query): Query<AuthorizeQuery>) {

}
