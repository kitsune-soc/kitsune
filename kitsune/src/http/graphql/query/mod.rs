use self::{account::AccountQuery, post::PostQuery};
use async_graphql::MergedObject;

mod account;
mod post;

#[derive(Default, MergedObject)]
pub struct RootQuery(AccountQuery, PostQuery);
