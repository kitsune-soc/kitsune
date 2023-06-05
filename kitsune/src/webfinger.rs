use crate::{consts::USER_AGENT, error::Result};
use autometrics::autometrics;
use http::HeaderValue;
use kitsune_cache::{ArcCache, CacheBackend, RedisCache};
use kitsune_http_client::Client;
use kitsune_type::webfinger::Resource;
use std::{sync::Arc, time::Duration};

const CACHE_DURATION: Duration = Duration::from_secs(10 * 60); // 10 minutes

#[derive(Clone)]
pub struct Webfinger {
    cache: ArcCache<str, String>,
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
    pub fn new(cache: ArcCache<str, String>) -> Self {
        Self {
            cache,
            client: Client::builder()
                .default_header("Accept", HeaderValue::from_static("application/jrd+json"))
                .unwrap()
                .user_agent(USER_AGENT)
                .unwrap()
                .build(),
        }
    }

    #[instrument(skip(self))]
    #[autometrics(track_concurrency)]
    pub async fn fetch_actor_url(&self, username: &str, domain: &str) -> Result<Option<String>> {
        let acct = format!("acct:{username}@{domain}");
        if let Some(ap_id) = self.cache.get(&acct).await? {
            return Ok(Some(ap_id));
        }

        let webfinger_url = format!("https://{domain}/.well-known/webfinger?resource={acct}");
        let resource: Resource = self.client.get(webfinger_url).await?.json().await?;
        let Some(ap_id) = resource
            .links
            .into_iter()
            .find_map(|link| (link.rel == "self").then_some(link.href?))
        else {
            return Ok(None);
        };
        self.cache.set(&acct, &ap_id).await?;

        Ok(Some(ap_id))
    }
}

#[cfg(test)]
mod test {
    use super::Webfinger;
    use kitsune_cache::NoopCache;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    #[tokio::test]
    async fn fetch_qarnax_ap_id() {
        let webfinger = Webfinger::new(Arc::new(NoopCache.into()));
        let ap_id = webfinger
            .fetch_actor_url("qarnax", "corteximplant.com")
            .await
            .expect("Failed to fetch resource");

        assert_eq!(ap_id, Some("https://corteximplant.com/users/qarnax".into()));
    }
}
