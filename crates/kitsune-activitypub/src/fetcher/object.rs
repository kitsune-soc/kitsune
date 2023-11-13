use super::Fetcher;
use crate::{error::Result, process_new_object, ProcessNewObject};
use async_recursion::async_recursion;
use autometrics::autometrics;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_cache::CacheBackend;
use kitsune_core::traits::Resolver;
use kitsune_db::{model::post::Post, schema::posts};
use kitsune_type::ap::Object;
use scoped_futures::ScopedFutureExt;

// Maximum call depth of fetching new posts. Prevents unbounded recursion.
// Setting this to >=40 would cause the `fetch_infinitely_long_reply_chain` test to run into stack overflow
pub const MAX_FETCH_DEPTH: u32 = 30;

impl<R> Fetcher<R>
where
    R: Resolver,
{
    #[async_recursion]
    pub(crate) async fn fetch_object_inner(
        &self,
        url: &str,
        call_depth: u32,
    ) -> Result<Option<Post>> {
        if call_depth > MAX_FETCH_DEPTH {
            return Ok(None);
        }

        if let Some(post) = self.post_cache.get(url).await? {
            return Ok(Some(post));
        }

        let post = self
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    posts::table
                        .filter(posts::url.eq(url))
                        .select(Post::as_select())
                        .first(db_conn)
                        .await
                        .optional()
                }
                .scoped()
            })
            .await?;

        if let Some(post) = post {
            self.post_cache.set(url, &post).await?;
            return Ok(Some(post));
        }

        let object: Object = self.fetch_ap_resource(url).await?;

        let process_data = ProcessNewObject::builder()
            .call_depth(call_depth)
            .db_pool(&self.db_pool)
            .embed_client(self.embed_client.as_ref())
            .fetcher(self)
            .object(Box::new(object))
            .search_backend(&self.search_backend)
            .build();
        let post = process_new_object(process_data).await?;

        self.post_cache.set(&post.url, &post).await?;

        Ok(Some(post))
    }

    #[instrument(skip(self))]
    #[autometrics(track_concurrency)]
    pub(crate) async fn fetch_object(&self, url: &str) -> Result<Post> {
        self.fetch_object_inner(url, 0)
            .await
            .transpose()
            .expect("[Bug] Highest level fetch returned a `None`")
    }
}
