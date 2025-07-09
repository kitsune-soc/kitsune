#[macro_use]
extern crate tracing;

use async_trait::async_trait;
use fred::clients::Pool as RedisPool;
use http::{HeaderValue, Request, StatusCode, header::ACCEPT};
use kitsune_cache::{ArcCache, CacheBackend, RedisCache};
use kitsune_core::traits::{Resolver, resolver::AccountResource};
use kitsune_error::Result;
use kitsune_http_client::Client;
use kitsune_type::webfinger::Resource;
use kitsune_util::try_join;
use std::{ptr, time::Duration};
use triomphe::Arc;

const CACHE_DURATION: Duration = Duration::from_secs(10 * 60); // 10 minutes
static ACCEPT_VALUE: HeaderValue = HeaderValue::from_static("application/jrd+json");

/// Intended to allow up to one canonicalisation on the originating server, one cross-origin
/// canonicalisation and one more canonicalisation on the destination server,
///
/// e.g. `acct:a@example.com -> acct:A@example.com -> acct:A@example.net -> a@example.net`
pub const MAX_JRD_REDIRECTS: u32 = 3;

#[derive(Clone)]
pub struct Webfinger {
    cache: ArcCache<str, AccountResource>,
    http_client: Client,
}

impl Webfinger {
    #[must_use]
    pub fn with_defaults(client: Client, redis_pool: RedisPool) -> Self {
        Self::new(
            client,
            Arc::new(RedisCache::new(redis_pool, "webfinger", CACHE_DURATION).into()),
        )
    }
}

impl Webfinger {
    #[must_use]
    pub fn new(client: Client, cache: ArcCache<str, AccountResource>) -> Self {
        Self {
            cache,
            http_client: client,
        }
    }
}

#[async_trait]
impl Resolver for Webfinger {
    /// Resolves the `acct:{username}@{domain}` URI via WebFinger to get the object ID and the
    /// canonical `acct:` URI of an ActivityPub actor
    ///
    /// This does *not* check that the resolved ActivityPub actor's
    /// `acct:{preferredUsername}@{domain}` URI points back to the resolved `acct:` resource,
    /// which the caller should check by themselves before trusting the result.
    #[cfg_attr(not(coverage), instrument(skip(self)))]
    async fn resolve_account(
        &self,
        username: &str,
        domain: &str,
    ) -> Result<Option<AccountResource>> {
        // XXX: Assigning the arguments to local bindings because the `#[cfg_attr(not(coverage), instrument)]` attribute
        // desugars to an `async move {}` block, inside which mutating the function arguments would
        // upset the borrowck
        // cf. <https://github.com/tokio-rs/tracing/issues/2717>
        let mut username = username;
        let mut domain = domain;

        let original_acct = format!("acct:{}@{domain}", urlencoding::encode(username));

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
            let request = Request::builder()
                .header(ACCEPT, &ACCEPT_VALUE)
                .uri(webfinger_url)
                .body(kitsune_http_client::Body::empty())
                .unwrap();

            let response = self.http_client.execute(request).await?;
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
        let cache_resolved_key_fut = async {
            if acct == original_acct {
                None
            } else {
                Some(self.cache.set(acct, &ret).await)
            }
            .transpose()
        };

        try_join!(cache_original_key_fut, cache_resolved_key_fut)?;

        Ok(Some(ret))
    }
}
