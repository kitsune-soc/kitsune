use crate::{consts::USER_AGENT, error::Result, try_join};
use autometrics::autometrics;
use core::ptr;
use futures_util::future::{FutureExt, OptionFuture};
use http::{HeaderValue, StatusCode};
use kitsune_cache::{ArcCache, CacheBackend, RedisCache};
use kitsune_http_client::Client;
use kitsune_type::webfinger::Resource;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

const CACHE_DURATION: Duration = Duration::from_secs(10 * 60); // 10 minutes
/// Intended to allow up to one canonicalisation on the originating server, one cross-origin
/// canonicalisation and one more canonicalisation on the destination server,
/// e.g. `acct:a@example.com -> acct:A@example.com -> acct:A@example.net -> a@example.net`
const MAX_JRD_REDIRECTS: u32 = 3;

#[derive(Clone)]
pub struct Webfinger {
    cache: ArcCache<str, ActorResource>,
    client: Client,
}

#[allow(clippy::doc_markdown)] // "WebFinger" here isn't referring to the item name
/// Description of an ActivityPub actor resolved via WebFinger
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActorResource {
    /// The `self` link (the actor's URI)
    pub uri: String,
    /// The username part of the canonical `acct:` URI
    pub username: String,
    /// The host component of the canonical `acct:` URI
    pub domain: String,
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
    pub fn new(cache: ArcCache<str, ActorResource>) -> Self {
        let client = Client::builder()
            .default_header("Accept", HeaderValue::from_static("application/jrd+json"))
            .unwrap()
            .user_agent(USER_AGENT)
            .unwrap()
            .build();
        Self::with_client(client, cache)
    }

    #[must_use]
    pub fn with_client(client: Client, cache: ArcCache<str, ActorResource>) -> Self {
        Self { cache, client }
    }

    /// Resolves the `acct:{username}@{domain}` URI via WebFinger to get the object ID and the
    /// canonical `acct:` URI of an ActivityPub actor
    ///
    /// This does *not* check that the resolved ActivityPub actor's
    /// `acct:{preferredUsername}@{domain}` URI points back to the resolved `acct:` resource,
    /// which the caller should check by themselves before trusting the result.
    #[instrument(skip(self))]
    #[autometrics(track_concurrency)]
    pub async fn resolve_actor(
        &self,
        username: &str,
        domain: &str,
    ) -> Result<Option<ActorResource>> {
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

        let ret = ActorResource {
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

#[cfg(test)]
mod test {
    use super::{Webfinger, MAX_JRD_REDIRECTS};
    use core::convert::Infallible;
    use hyper::{Body, Request, Response, StatusCode};
    use kitsune_cache::NoopCache;
    use kitsune_http_client::Client;
    use kitsune_type::webfinger::Resource;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;
    use tower::service_fn;

    #[tokio::test]
    async fn basic() {
        let client = service_fn(|req: Request<_>| async move {
            assert_eq!(
                req.uri().path_and_query().unwrap(),
                "/.well-known/webfinger?resource=acct:0x0@corteximplant.com"
            );
            let body = include_str!("../../../test-fixtures/0x0_jrd.json");
            Ok::<_, Infallible>(Response::new(Body::from(body)))
        });
        let client = Client::builder().service(client);

        let webfinger = Webfinger::with_client(client, Arc::new(NoopCache.into()));
        let resource = webfinger
            .resolve_actor("0x0", "corteximplant.com")
            .await
            .expect("Failed to fetch resource")
            .unwrap();

        assert_eq!(resource.username, "0x0");
        assert_eq!(resource.domain, "corteximplant.com");
        assert_eq!(resource.uri, "https://corteximplant.com/users/0x0");
    }

    #[tokio::test]
    async fn follow_jrd_redirect() {
        let mut base = include_bytes!("../../../test-fixtures/0x0_jrd.json").to_owned();
        let body = simd_json::to_string(&Resource {
            subject: "acct:0x0@joinkitsune.org".into(),
            ..simd_json::from_slice(&mut base).unwrap()
        })
        .unwrap();
        let client = service_fn(move |req: Request<_>| {
            let body = body.clone();
            async move {
                match (
                    req.uri().authority().unwrap().as_str(),
                    req.uri().path_and_query().unwrap().as_str(),
                ) {
                    (
                        "corteximplant.com",
                        "/.well-known/webfinger?resource=acct:0x0@corteximplant.com",
                    )
                    | (
                        "joinkitsune.org",
                        "/.well-known/webfinger?resource=acct:0x0@joinkitsune.org",
                    ) => Ok::<_, Infallible>(Response::new(Body::from(body))),
                    _ => panic!("HTTP client hit unexpected route: {}", req.uri()),
                }
            }
        });
        let client = Client::builder().service(client);

        let webfinger = Webfinger::with_client(client, Arc::new(NoopCache.into()));
        let resource = webfinger
            .resolve_actor("0x0", "corteximplant.com")
            .await
            .expect("Failed to fetch resource")
            .unwrap();

        assert_eq!(resource.username, "0x0");
        assert_eq!(resource.domain, "joinkitsune.org");
        assert_eq!(resource.uri, "https://corteximplant.com/users/0x0");
    }

    #[tokio::test]
    async fn reject_fake_jrd_redirect() {
        let client = service_fn(|req: Request<_>| async move {
            match (
                req.uri().authority().unwrap().as_str(),
                req.uri().path_and_query().unwrap().as_str(),
            ) {
                (
                    "corteximplant.com",
                    "/.well-known/webfinger?resource=acct:0x0@corteximplant.com",
                ) => {
                    let mut base = include_bytes!("../../../test-fixtures/0x0_jrd.json").to_owned();
                    let body = simd_json::to_string(&Resource {
                        subject: "acct:0x0@whitehouse.gov".into(),
                        ..simd_json::from_slice(&mut base).unwrap()
                    })
                    .unwrap();
                    Ok::<_, Infallible>(Response::new(Body::from(body)))
                }
                ("whitehouse.gov", "/.well-known/webfinger?resource=acct:0x0@whitehouse.gov") => {
                    Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap())
                }
                _ => panic!("HTTP client hit unexpected route: {}", req.uri()),
            }
        });
        let client = Client::builder().service(client);

        let webfinger = Webfinger::with_client(client, Arc::new(NoopCache.into()));
        let resource = webfinger
            .resolve_actor("0x0", "corteximplant.com")
            .await
            .expect("Failed to fetch resource");

        assert!(resource.is_none(), "resource = {resource:?}");
    }

    #[tokio::test]
    async fn reject_unbounded_number_of_jrd_redirects() {
        let client = service_fn(|req: Request<_>| async move {
            let Some(count) = req
                .uri()
                .path_and_query()
                .unwrap()
                .as_str()
                .strip_prefix("/.well-known/webfinger?resource=acct:0x")
                .and_then(|suffix| suffix.strip_suffix("@corteximplant.com"))
                .and_then(|count| u32::from_str_radix(count, 16).ok())
            else {
                panic!(
                    "HTTP client hit unexpected route: {}",
                    req.uri().path_and_query().unwrap()
                );
            };
            assert!(count <= MAX_JRD_REDIRECTS);
            let mut base = include_bytes!("../../../test-fixtures/0x0_jrd.json").to_owned();
            let body = simd_json::to_string(&Resource {
                subject: format!("acct:0x{:x}@corteximplant.com", count + 1),
                ..simd_json::from_slice(&mut base).unwrap()
            })
            .unwrap();
            Ok::<_, Infallible>(Response::new(Body::from(body)))
        });
        let client = Client::builder().service(client);

        let webfinger = Webfinger::with_client(client, Arc::new(NoopCache.into()));
        let resource = webfinger
            .resolve_actor("0x0", "corteximplant.com")
            .await
            .expect("Failed to fetch resource");

        assert!(resource.is_none(), "resource = {resource:?}");
    }
}
