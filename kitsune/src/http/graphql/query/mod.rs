use self::{
    account::AccountQuery, instance::InstanceQuery, post::PostQuery, timeline::TimelineQuery,
};
use async_graphql::MergedObject;

mod account;
mod instance;
mod post;
mod timeline;

#[derive(Default, MergedObject)]
pub struct RootQuery(InstanceQuery, AccountQuery, PostQuery, TimelineQuery);
