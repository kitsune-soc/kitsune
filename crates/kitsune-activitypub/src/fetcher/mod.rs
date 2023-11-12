use crate::{
    error::{Error, Result},
    webfinger::Webfinger,
};
use headers::{ContentType, HeaderMapExt};
use http::HeaderValue;
use kitsune_cache::ArcCache;
use kitsune_consts::USER_AGENT;
use kitsune_db::{
    model::{account::Account, post::Post},
    PgPool,
};
use kitsune_embed::Client as EmbedClient;
use kitsune_federation_filter::FederationFilter;
use kitsune_http_client::Client;
use kitsune_type::jsonld::RdfNode;
use mime::Mime;
use serde::de::DeserializeOwned;
use typed_builder::TypedBuilder;
use url::Url;

mod actor;
mod emoji;
mod object;

#[derive(Clone, Debug, TypedBuilder)]
/// Options passed to the fetcher
pub struct FetchOptions<'a> {
    /// Prefetched WebFinger `acct` URI
    #[builder(default, setter(strip_option))]
    acct: Option<(&'a str, &'a str)>,

    /// Refetch the ActivityPub entity
    ///
    /// This is mainly used to refresh possibly stale actors
    ///
    /// Default: false
    #[builder(default = false)]
    refetch: bool,

    /// URL of the ActivityPub entity
    url: &'a str,
}

impl<'a> From<&'a str> for FetchOptions<'a> {
    fn from(value: &'a str) -> Self {
        Self::builder().url(value).build()
    }
}

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
    webfinger: Webfinger,

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
            return Err(Error::InvalidResponse.into());
        }

        Ok(response.jsonld().await?)
    }
}
