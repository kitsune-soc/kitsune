use crate::error::{Error, Result};
use headers::{ContentType, HeaderMapExt};
use http::HeaderValue;
use kitsune_cache::ArcCache;
use kitsune_consts::USER_AGENT;
use kitsune_core::traits::{fetcher::AccountFetchOptions, Fetcher as FetcherTrait};
use kitsune_db::{
    model::{account::Account, custom_emoji::CustomEmoji, post::Post},
    PgPool,
};
use kitsune_embed::Client as EmbedClient;
use kitsune_federation_filter::FederationFilter;
use kitsune_http_client::Client;
use kitsune_type::jsonld::RdfNode;
use kitsune_webfinger::Webfinger;
use mime::Mime;
use serde::de::DeserializeOwned;
use typed_builder::TypedBuilder;
use url::Url;

pub use self::object::MAX_FETCH_DEPTH;

mod actor;
mod emoji;
mod object;

#[derive(Clone, TypedBuilder)]
pub struct Fetcher {
    #[builder(default =
        Client::builder()
            .default_header(
                "accept",
                HeaderValue::from_static(
                    "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\", application/activity+json",
                ),
            )
            .unwrap()
            .user_agent(USER_AGENT)
            .unwrap()
            .build()
    )]
    client: Client,
    db_pool: PgPool,
    embed_client: Option<EmbedClient>,
    federation_filter: FederationFilter,
    #[builder(setter(into))]
    search_backend: kitsune_search::AnySearchBackend,
    resolver: Webfinger,

    // Caches
    post_cache: ArcCache<str, Post>,
    user_cache: ArcCache<str, Account>,
}

impl Fetcher {
    async fn fetch_ap_resource<U, T>(&self, url: U) -> Result<T>
    where
        U: TryInto<Url>,
        Error: From<<U as TryInto<Url>>::Error>,
        T: DeserializeOwned + RdfNode,
    {
        let url = url.try_into()?;
        if !self.federation_filter.is_url_allowed(&url)? {
            return Err(Error::BlockedInstance);
        }

        let response = self.client.get(url.as_str()).await?;
        let Some(content_type) = response
            .headers()
            .typed_get::<ContentType>()
            .map(Mime::from)
        else {
            return Err(Error::InvalidResponse);
        };

        let is_json_ld_activitystreams = || {
            content_type
                .essence_str()
                .eq_ignore_ascii_case("application/ld+json")
                && content_type
                    .get_param("profile")
                    .map_or(false, |profile_urls| {
                        profile_urls
                            .as_str()
                            .split_whitespace()
                            .any(|url| url == "https://www.w3.org/ns/activitystreams")
                    })
        };

        let is_activity_json = || {
            content_type
                .essence_str()
                .eq_ignore_ascii_case("application/activity+json")
        };

        if !is_json_ld_activitystreams() && !is_activity_json() {
            return Err(Error::InvalidResponse);
        }

        Ok(response.jsonld().await?)
    }
}

impl FetcherTrait for Fetcher {
    type Error = Error;

    async fn fetch_account(&self, opts: AccountFetchOptions<'_>) -> Result<Account, Self::Error> {
        self.fetch_actor(opts).await
    }

    async fn fetch_emoji(&self, url: &str) -> Result<CustomEmoji, Self::Error> {
        self.fetch_emoji(url).await
    }

    async fn fetch_post(&self, url: &str) -> Result<Post, Self::Error> {
        self.fetch_object(url).await
    }
}
