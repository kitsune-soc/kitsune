use crate::{consts::USER_AGENT, error::Result};
use http::{HeaderMap, HeaderValue};
use redis::AsyncCommands;
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
    client: Client,
    redis_conn: deadpool_redis::Pool,
}

impl Webfinger {
    pub fn new(redis_conn: deadpool_redis::Pool) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/jrd+json"));

        Self {
            client: Client::builder()
                .default_headers(headers)
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
            redis_conn,
        }
    }

    pub async fn fetch_actor_url(&self, username: &str, domain: &str) -> Result<Option<String>> {
        let mut conn = self.redis_conn.get().await?;
        let acct = format!("acct:{username}@{domain}");

        let cache_key = format!("webfinger:{acct}");
        if let Some(ap_id) = conn.get(&cache_key).await? {
            Ok(Some(ap_id))
        } else {
            let webfinger_url = format!("https://{domain}/.well-known/webfinger?resource={acct}");

            let resource: Resource = self.client.get(webfinger_url).send().await?.json().await?;
            let Some(ap_id) = resource
                .links
                .into_iter()
                .find_map(|link| (link.rel == "self").then_some(link.href))
            else {
                return Ok(None);
            };

            #[allow(clippy::cast_possible_truncation)]
            conn.set_ex(cache_key, &ap_id, CACHE_DURATION.as_secs() as usize)
                .await?;

            Ok(Some(ap_id))
        }
    }
}
