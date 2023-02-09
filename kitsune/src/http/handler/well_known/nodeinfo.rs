use crate::state::Zustand;
use axum::{extract::State, routing, Json, Router};
use kitsune_type::nodeinfo::well_known::{Link, WellKnown};

#[allow(clippy::unused_async)]
async fn get(State(zustand): State<Zustand>) -> Json<WellKnown> {
    let href = format!("https://{}/nodeinfo/2.1", zustand.config.domain);

    Json(WellKnown {
        links: vec![Link {
            rel: kitsune_type::nodeinfo::well_known::Rel::TwoOne,
            href,
        }],
    })
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
