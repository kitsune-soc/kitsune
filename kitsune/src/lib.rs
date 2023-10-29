#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    forbidden_lint_groups
)]

#[macro_use]
extern crate metrics;

#[macro_use]
extern crate tracing;

pub mod consts;
pub mod error;
pub mod http;
pub mod oauth2;
pub mod state;

use self::{
    oauth2::{OAuth2Service, OAuthEndpoint},
    state::{SessionConfig, Zustand},
};
use athena::JobQueue;
use kitsune_config::Configuration;
use kitsune_core::job::KitsuneContextRepo;
use kitsune_db::PgPool;

#[cfg(feature = "oidc")]
use {futures_util::future::OptionFuture, kitsune_oidc::OidcService};

pub async fn initialise_state(
    config: &Configuration,
    conn: PgPool,
    job_queue: JobQueue<KitsuneContextRepo>,
) -> eyre::Result<Zustand> {
    let core_state = kitsune_core::prepare_state(config, conn.clone(), job_queue).await?;

    #[cfg(feature = "oidc")]
    let oidc_service = OptionFuture::from(config.server.oidc.as_ref().map(|oidc_config| {
        OidcService::initialise(oidc_config, core_state.service.url.oidc_redirect_uri())
    }))
    .await
    .transpose()?;

    let oauth2_service = OAuth2Service::builder()
        .db_pool(conn.clone())
        .url_service(core_state.service.url.clone())
        .build();

    Ok(Zustand {
        core: core_state,
        oauth2: oauth2_service,
        oauth_endpoint: OAuthEndpoint::from(conn),
        #[cfg(feature = "oidc")]
        oidc: oidc_service,
        session_config: SessionConfig::generate(),
    })
}
