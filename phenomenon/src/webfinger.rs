use crate::{consts::USER_AGENT, error::Result};
use http::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Link {
    pub rel: String,
    pub href: Option<String>,
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
            .find_map(|link| (link.rel == "self").then_some(link.href?)))
    }
}

#[cfg(test)]
mod test {
    use super::{Resource, Webfinger};

    const GARGRON_WEBFINGER_RESOURCE: &str = r#"
    {
        "subject": "acct:Gargron@mastodon.social",
        "aliases": [
            "https://mastodon.social/@Gargron",
            "https://mastodon.social/users/Gargron"
        ],
        "links": [
            {
                "rel": "http://webfinger.net/rel/profile-page",
                "type": "text/html",
                "href": "https://mastodon.social/@Gargron"
            },
            {
                "rel": "self",
                "type": "application/activity+json",
                "href": "https://mastodon.social/users/Gargron"
            },
            {
                "rel": "http://ostatus.org/schema/1.0/subscribe",
                "template": "https://mastodon.social/authorize_interaction?uri={uri}"
            }
        ]
    }
    "#;

    #[test]
    fn deserialise_gargron() {
        let deserialised: Resource = serde_json::from_str(GARGRON_WEBFINGER_RESOURCE)
            .expect("Failed to deserialise resource");

        assert_eq!(deserialised.subject, "acct:Gargron@mastodon.social");
        assert_eq!(
            deserialised.aliases,
            [
                "https://mastodon.social/@Gargron",
                "https://mastodon.social/users/Gargron"
            ]
        );
    }

    #[tokio::test]
    async fn fetch_qarnax_ap_id() {
        let webfinger = Webfinger::new();
        let ap_id = webfinger
            .fetch_actor_url("qarnax", "corteximplant.com")
            .await
            .expect("Failed to fetch resource");

        assert_eq!(ap_id, Some("https://corteximplant.com/users/qarnax".into()));
    }
}
