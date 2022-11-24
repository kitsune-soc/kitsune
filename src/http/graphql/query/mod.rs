use self::{instance::InstanceQuery, post::PostQuery, user::UserQuery};
use async_graphql::MergedObject;

mod instance;
mod post;
mod user;

#[derive(Default, MergedObject)]
pub struct RootQuery(InstanceQuery, PostQuery, UserQuery);
