use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    ActivityPub,
    Buddycloud,
    Dfrn,
    Diaspora,
    Libertree,
    OStatus,
    PumpIo,
    Tent,
    Xmpp,
    Zot,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum InboundService {
    #[serde(rename = "atom1.0")]
    Atom1_0,
    GnuSocial,
    Imap,
    Pnut,
    Pop3,
    PumpIo,
    #[serde(rename = "rss2.0")]
    Rss2_0,
    Twitter,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutboundService {
    #[serde(rename = "atom1.0")]
    Atom1_0,
    Blogger,
    Buddycloud,
    Diaspora,
    Dreamwidth,
    Drupal,
    Facebook,
    Friendica,
    GnuSocial,
    Google,
    InsaneJournal,
    Libertree,
    LinkedIn,
    LiveJournal,
    Mediagoblin,
    MySpace,
    Pinterest,
    Pnut,
    Posterous,
    PumpIo,
    Redmatrix,
    #[serde(rename = "rss2.0")]
    Rss2_0,
    Smtp,
    Tent,
    Tumblr,
    Twitter,
    Wordpress,
    Xmpp,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, ToSchema)]
pub enum Version {
    #[serde(rename = "2.1")]
    TwoOne,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Software {
    pub name: String,
    pub version: String,
    pub repository: String,
    pub homepage: Option<String>,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Services {
    pub inbound: Vec<InboundService>,
    pub outbound: Vec<OutboundService>,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsageUsers {
    pub total: u64,
    pub active_halfyear: Option<u64>,
    pub active_month: Option<u64>,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub users: UsageUsers,
    pub local_posts: u64,
    pub local_comments: Option<u64>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
/// Definitions of Nodeinfo 2.1
pub struct TwoOne {
    pub version: Version,
    pub software: Software,
    pub protocols: Vec<Protocol>,
    pub services: Services,
    pub open_registrations: bool,
    pub usage: Usage,
    #[schema(value_type = Object)]
    pub metadata: Value,
}

#[cfg(test)]
mod test {
    use super::TwoOne;
    use crate::nodeinfo::two_one::Protocol;
    use pretty_assertions::{assert_eq, assert_str_eq};

    #[test]
    fn deserialize_akkoma_nodeinfo() {
        let raw = include_str!("../../tests/nodeinfo_2.1.json");
        let two_one: TwoOne = serde_json::from_str(raw).unwrap();

        assert_str_eq!(two_one.software.name, "akkoma");
        assert_eq!(two_one.protocols, [Protocol::ActivityPub]);
    }
}
