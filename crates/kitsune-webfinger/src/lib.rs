#[macro_use]
extern crate tracing;

use crate::error::{Error, Result};
use autometrics::autometrics;
use futures_util::future::{FutureExt, OptionFuture};
use http::{HeaderValue, StatusCode};
use kitsune_cache::{ArcCache, CacheBackend, RedisCache};
use kitsune_consts::USER_AGENT;
use kitsune_core::traits::{resolver::AccountResource, Resolver};
use kitsune_http_client::Client;
use kitsune_type::webfinger::Resource;
use kitsune_util::try_join;
use std::{ptr, sync::Arc, time::Duration};

pub mod error;

const CACHE_DURATION: Duration = Duration::from_secs(10 * 60); // 10 minutes

/// Intended to allow up to one canonicalisation on the originating server, one cross-origin
/// canonicalisation and one more canonicalisation on the destination server,
/// e.g. `acct:a@example.com -> acct:A@example.com -> acct:A@example.net -> a@example.net`
pub const MAX_JRD_REDIRECTS: u32 = 3;

#[derive(Clone)]
pub struct Webfinger {
    cache: ArcCache<str, AccountResource>,
    client: Client,
}

impl Webfinger {
    #[must_use]
    pub fn with_defaults(redis_conn: deadpool_redis::Pool) -> Self {
        Self::new(Arc::new(
            RedisCache::new(redis_conn, "webfinger", CACHE_DURATION).into(),
        ))
    }
}

impl Webfinger {
    #[allow(clippy::missing_panics_doc)] // The invariants are covered. It won't panic.
    #[must_use]
    pub fn new(cache: ArcCache<str, AccountResource>) -> Self {
        let client = Client::builder()
            .default_header("Accept", HeaderValue::from_static("application/jrd+json"))
            .unwrap()
            .user_agent(USER_AGENT)
            .unwrap()
            .build();

        Self::with_client(client, cache)
    }

    #[must_use]
    pub fn with_client(client: Client, cache: ArcCache<str, AccountResource>) -> Self {
        Self { cache, client }
    }
}

impl Resolver for Webfinger {
    type Error = Error;

    /// Resolves the `acct:{username}@{domain}` URI via WebFinger to get the object ID and the
    /// canonical `acct:` URI of an ActivityPub actor
    ///
    /// This does *not* check that the resolved ActivityPub actor's
    /// `acct:{preferredUsername}@{domain}` URI points back to the resolved `acct:` resource,
    /// which the caller should check by themselves before trusting the result.
    #[instrument(skip(self))]
    #[autometrics(track_concurrency)]
    async fn resolve_account(
        &self,
        username: &str,
        domain: &str,
    ) -> Result<Option<AccountResource>, Self::Error> {
        // XXX: Assigning the arguments to local bindings because the `#[instrument]` attribute
        // desugars to an `async move {}` block, inside which mutating the function arguments would
        // upset the borrowck
        // cf. <https://github.com/tokio-rs/tracing/issues/2717>
        let mut username = username;
        let mut domain = domain;

        let original_acct = format!("acct:{username}@{domain}");

        let mut acct_buf: String;
        let mut acct = original_acct.as_str();
        let mut remaining_redirects = MAX_JRD_REDIRECTS;
        let links = loop {
            if let Some(ret) = self.cache.get(acct).await? {
                if !ptr::eq(acct, original_acct.as_str()) {
                    self.cache.set(&original_acct, &ret).await?;
                }
                return Ok(Some(ret));
            }

            let webfinger_url = format!("https://{domain}/.well-known/webfinger?resource={acct}");
            let response = self.client.get(webfinger_url).await?;

            if matches!(response.status(), StatusCode::NOT_FOUND | StatusCode::GONE) {
                // Either the actor couldn't be found or the server doesn't support WebFinger
                return Ok(None);
            }

            let resource: Resource = response.json().await?;

            if resource.subject == acct {
                break resource.links;
            }

            // Prepare another query to resolve the new subject

            if remaining_redirects == 0 {
                return Ok(None);
            }

            acct_buf = resource.subject;
            acct = &acct_buf;

            let Some(username_domain) = acct
                .strip_prefix("acct:")
                .and_then(|acct| acct.split_once('@'))
            else {
                return Ok(None);
            };

            (username, domain) = username_domain;

            remaining_redirects -= 1;
        };

        let Some(uri) = links
            .into_iter()
            .find_map(|link| (link.rel == "self").then_some(link.href?))
        else {
            return Ok(None);
        };

        let ret = AccountResource {
            username: username.to_owned(),
            domain: domain.to_owned(),
            uri,
        };

        let cache_original_key_fut = self.cache.set(&original_acct, &ret);
        let cache_resolved_key_fut =
            OptionFuture::from((acct != original_acct).then(|| self.cache.set(acct, &ret)))
                .map(Option::transpose);

        try_join!(cache_original_key_fut, cache_resolved_key_fut)?;

        Ok(Some(ret))
    }
}
