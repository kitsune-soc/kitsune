use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct Instance {
    pub description: String,
    pub domain: String,
    pub local_post_count: u64,
    pub name: String,
    pub registrations_open: bool,
    pub user_count: u64,
    pub version: &'static str,
}
