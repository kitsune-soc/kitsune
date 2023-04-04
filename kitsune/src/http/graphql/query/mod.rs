use self::{account::AccountQuery, instance::InstanceQuery, post::PostQuery};
use async_graphql::MergedObject;

mod account;
mod instance;
mod post;

#[derive(Default, MergedObject)]
pub struct RootQuery(InstanceQuery, AccountQuery, PostQuery);
