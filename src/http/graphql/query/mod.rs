use self::{auth::AuthQuery, post::PostQuery, user::UserQuery};
use async_graphql::MergedObject;

mod auth;
mod post;
mod user;

#[derive(Default, MergedObject)]
pub struct RootQuery(AuthQuery, PostQuery, UserQuery);
