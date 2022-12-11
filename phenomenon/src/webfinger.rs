use crate::{cache::Cacher, consts::USER_AGENT, error::Result};
use http::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const CACHE_DURATION: Duration = Duration::from_secs(10 * 60); // 10 minutes

#[derive(Deserialize, Serialize)]
pub struct Link {
    pub rel: String,
    pub href: String,
}

#[derive(Deserialize, Serialize)]
pub struct Resource {
    pub subject: String,
    pub aliases: Vec<String>,
    pub links: Vec<Link>,
}

#[derive(Clone)]
pub struct Webfinger {
    cacher: Cacher<String, String>,
    client: Client,
}

impl Webfinger {
    #[allow(clippy::missing_panics_doc)] // The invariants are covered. It won't panic.
    #[must_use]
    pub fn new(redis_conn: deadpool_redis::Pool) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/jrd+json"));

        Self {
            cacher: Cacher::new(redis_conn, "webfinger", CACHE_DURATION),
            client: Client::builder()
                .default_headers(headers)
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
        }
    }

    pub async fn fetch_actor_url(&self, username: &str, domain: &str) -> Result<Option<String>> {
        let acct = format!("acct:{username}@{domain}");
        if let Some(ap_id) = self.cacher.get(&acct).await? {
            return Ok(Some(ap_id));
        }

        let webfinger_url = format!("https://{domain}/.well-known/webfinger?resource={acct}");
        let resource: Resource = self.client.get(webfinger_url).send().await?.json().await?;
        let Some(ap_id) = resource
            .links
            .into_iter()
            .find_map(|link| (link.rel == "self").then_some(link.href))
        else {
            return Ok(None);
        };
        self.cacher.set(&acct, &ap_id).await?;

        Ok(Some(ap_id))
    }
}
