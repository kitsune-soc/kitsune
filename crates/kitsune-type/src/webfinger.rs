use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Link {
    pub rel: String,
    pub r#type: Option<String>,
    pub href: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Resource {
    pub subject: String,
    pub aliases: Vec<String>,
    pub links: Vec<Link>,
}

#[cfg(test)]
mod test {
    use crate::webfinger::Resource;
    use pretty_assertions::assert_eq;

    const GARGRON_WEBFINGER_RESOURCE: &[u8] = br#"
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
        let mut webfinger_resource = GARGRON_WEBFINGER_RESOURCE.to_vec();
        let deserialised: Resource =
            simd_json::from_slice(&mut webfinger_resource).expect("Failed to deserialise resource");

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
