use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Rel {
    #[serde(rename = "http://nodeinfo.diaspora.software/ns/schema/2.1")]
    TwoOne,
}

#[derive(Debug, Serialize)]
pub struct Link {
    pub rel: Rel,
    pub href: String,
}

#[derive(Debug, Serialize)]
pub struct WellKnown {
    pub links: Vec<Link>,
}
