use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub enum Rel {
    #[serde(rename = "http://nodeinfo.diaspora.software/ns/schema/2.1")]
    TwoOne,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Link {
    pub rel: Rel,
    pub href: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WellKnown {
    pub links: Vec<Link>,
}
