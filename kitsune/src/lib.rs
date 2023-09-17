#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    forbidden_lint_groups
)]

#[cfg(feature = "metrics")]
#[macro_use]
extern crate metrics;

#[macro_use]
extern crate tracing;

pub mod consts;
pub mod error;
pub mod http;
pub mod oauth2;
#[cfg(feature = "oidc")]
pub mod oidc;
pub mod state;

use self::{
    oauth2::OAuth2Service,
    state::{SessionConfig, Zustand},
};
use athena::JobQueue;
use eyre::Context;
use kitsune_core::{config::Configuration, job::KitsuneContextRepo};
use kitsune_db::PgPool;
use oauth2::OAuthEndpoint;

#[cfg(feature = "oidc")]
use {
    self::oidc::{async_client, OidcService},
    futures_util::future::OptionFuture,
    kitsune_core::{config::OidcConfiguration, service::url::UrlService},
    openidconnect::{
        core::{CoreClient, CoreProviderMetadata},
        ClientId, ClientSecret, IssuerUrl, RedirectUrl,
    },
};

#[cfg(feature = "oidc")]
async fn prepare_oidc_client(
    oidc_config: &OidcConfiguration,
    url_service: &UrlService,
) -> eyre::Result<CoreClient> {
    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(oidc_config.server_url.to_string()).context("Invalid OIDC issuer URL")?,
        async_client,
    )
    .await
    .context("Couldn't discover the OIDC provider metadata")?;

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(oidc_config.client_id.to_string()),
        Some(ClientSecret::new(oidc_config.client_secret.to_string())),
    )
    .set_redirect_uri(RedirectUrl::new(url_service.oidc_redirect_uri())?);

    Ok(client)
}

pub async fn initialise_state(
    config: &Configuration,
    conn: PgPool,
    job_queue: JobQueue<KitsuneContextRepo>,
) -> eyre::Result<Zustand> {
    let core_state = kitsune_core::prepare_state(config, conn.clone(), job_queue).await?;

    #[cfg(feature = "oidc")]
    let oidc_service = OptionFuture::from(config.server.oidc.as_ref().map(|oidc_config| async {
        let service = OidcService::builder()
            .client(prepare_oidc_client(oidc_config, &core_state.service.url).await?)
            .login_state(kitsune_core::prepare_cache(config, "OIDC-LOGIN-STATE")) // TODO: REPLACE THIS WITH A BETTER ALTERNATIVE TO JUST ABUSING A CACHE
            .build();

        Ok::<_, eyre::Report>(service)
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
