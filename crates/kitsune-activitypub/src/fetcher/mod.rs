use async_trait::async_trait;
use headers::{ContentType, HeaderMapExt};
use http::HeaderValue;
use kitsune_cache::ArcCache;
use kitsune_config::language_detection::Configuration as LanguageDetectionConfig;
use kitsune_core::{
    consts::USER_AGENT,
    traits::{
        coerce::CoerceResolver,
        fetcher::{AccountFetchOptions, PostFetchOptions},
        Fetcher as FetcherTrait, Resolver,
    },
};
use kitsune_db::{
    model::{account::Account, custom_emoji::CustomEmoji, post::Post},
    PgPool,
};
use kitsune_embed::Client as EmbedClient;
use kitsune_error::{bail, Error, Result};
use kitsune_federation_filter::FederationFilter;
use kitsune_http_client::Client;
use kitsune_type::jsonld::RdfNode;
use mime::Mime;
use serde::de::DeserializeOwned;
use triomphe::Arc;
use typed_builder::TypedBuilder;
use url::Url;

pub use self::object::MAX_FETCH_DEPTH;

mod actor;
mod emoji;
mod object;

#[derive(TypedBuilder)]
#[builder(build_method(into = Arc<Fetcher>))]
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
    language_detection_config: LanguageDetectionConfig,
    #[builder(setter(into))]
    search_backend: kitsune_search::AnySearchBackend,
    resolver: Arc<dyn Resolver>,

    // Caches
    account_cache: ArcCache<str, Account>,
    post_cache: ArcCache<str, Post>,
}

impl Fetcher {
    async fn fetch_ap_resource<U, T>(&self, url: U) -> Result<Option<T>>
    where
        U: TryInto<Url>,
        Error: From<<U as TryInto<Url>>::Error>,
        T: DeserializeOwned + RdfNode,
    {
        let url = url.try_into()?;
        if !self.federation_filter.is_url_allowed(&url)? {
            bail!("instance is blocked");
        }

        let response = self.client.get(url.as_str()).await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let Some(content_type) = response
            .headers()
            .typed_get::<ContentType>()
            .map(Mime::from)
        else {
            bail!("invalid content-type header in response");
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
            bail!("invalid content-type: isnt either ld+json or activity+json");
        }

        let response = response.jsonld().await?;

        Ok(Some(response))
    }
}

#[async_trait]
impl FetcherTrait for Fetcher {
    fn resolver(&self) -> Arc<dyn Resolver> {
        Arc::new(self.resolver.clone()).coerce()
    }

    async fn fetch_account(&self, opts: AccountFetchOptions<'_>) -> Result<Option<Account>> {
        Ok(self.fetch_actor(opts).await?)
    }

    async fn fetch_emoji(&self, url: &str) -> Result<Option<CustomEmoji>> {
        Ok(self.fetch_emoji(url).await?)
    }

    async fn fetch_post(&self, opts: PostFetchOptions<'_>) -> Result<Option<Post>> {
        Ok(self.fetch_object(opts.url, opts.call_depth).await?)
    }
}
