use self::{posts::PostQuery, users::UserQuery};
use async_graphql::MergedObject;

mod post;
mod user;

#[derive(Default, MergedObject)]
pub struct RootQuery(PostQuery, UserQuery);
