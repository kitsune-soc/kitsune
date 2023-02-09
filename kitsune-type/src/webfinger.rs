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

#[cfg(test)]
mod test {
    use crate::webfinger::Resource;
    use pretty_assertions::assert_eq;

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
}
