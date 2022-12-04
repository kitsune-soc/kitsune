use crate::{consts::USER_AGENT, error::Result};
use http::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};

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
}

impl Webfinger {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/jrd+json"));

        Self {
            client: Client::builder()
                .default_headers(headers)
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
        }
    }

    pub async fn fetch_actor_url(&self, username: &str, domain: &str) -> Result<Option<String>> {
        let acct = format!("acct:{username}@{domain}");
        let webfinger_url = format!("https://{domain}/.well-known/webfinger?resource={acct}");

        let resource: Resource = self.client.get(webfinger_url).send().await?.json().await?;
        Ok(resource
            .links
            .into_iter()
            .find_map(|link| (link.rel == "self").then_some(link.href)))
    }
}
