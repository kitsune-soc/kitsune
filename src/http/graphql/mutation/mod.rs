use self::{auth::AuthMutation, post::PostMutation};
use async_graphql::MergedObject;

mod auth;
mod post;

#[derive(Default, MergedObject)]
pub struct RootMutation(AuthMutation, PostMutation);
