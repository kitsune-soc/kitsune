use crate::http::graphql::{types::Post, ContextExt};
use async_graphql::{
    connection::{self, Connection, Edge},
    Context, Object, Result,
};
use futures_util::TryStreamExt;
use kitsune_core::service::timeline::{GetHome, GetPublic};
use speedy_uuid::Uuid;

#[derive(Default)]
pub struct TimelineQuery;

#[Object]
impl TimelineQuery {
    pub async fn home_timeline(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<Uuid, Post>> {
        let timeline_service = &ctx.state().service.timeline;

        connection::query(
            after,
            before,
            first,
            last,
            |after, before, first, _last| async move {
                let get_home = GetHome::builder()
                    .fetching_account_id(ctx.user_data()?.account.id)
                    .max_id(after)
                    .min_id(before);
                let get_home = if let Some(first) = first {
                    get_home.limit(first).build()
                } else {
                    get_home.build()
                };

                let mut post_stream = timeline_service
                    .get_home(get_home)
                    .await?
                    .map_ok(Post::from);

                let mut connection = Connection::new(true, true); // TODO: Set actual values
                while let Some(post) = post_stream.try_next().await? {
                    connection.edges.push(Edge::new(post.id, post));
                }

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    pub async fn public_timeline(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = true)] only_local: bool,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<Uuid, Post>> {
        let timeline_service = &ctx.state().service.timeline;

        connection::query(
            after,
            before,
            first,
            last,
            |after, before, first, _last| async move {
                let get_public = GetPublic::builder()
                    .max_id(after)
                    .min_id(before)
                    .only_local(only_local);
                let get_public = if let Some(first) = first {
                    get_public.limit(first).build()
                } else {
                    get_public.build()
                };

                let mut post_stream = timeline_service
                    .get_public(get_public)
                    .await?
                    .map_ok(Post::from);

                let mut connection = Connection::new(true, true); // TODO: Set actual values
                while let Some(post) = post_stream.try_next().await? {
                    connection.edges.push(Edge::new(post.id, post));
                }

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}
