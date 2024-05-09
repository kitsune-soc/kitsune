use super::Fetcher;
use crate::{process_new_object, ProcessNewObject};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_cache::CacheBackend;
use kitsune_db::{model::post::Post, schema::posts, with_connection};
use kitsune_error::Result;

// Maximum call depth of fetching new posts. Prevents unbounded recursion.
// Setting this to >=40 would cause the `fetch_infinitely_long_reply_chain` test to run into stack overflow
pub const MAX_FETCH_DEPTH: u32 = 15;

impl Fetcher {
    #[instrument(skip(self))]
    pub(crate) async fn fetch_object(&self, url: &str, call_depth: u32) -> Result<Option<Post>> {
        if call_depth > MAX_FETCH_DEPTH {
            return Ok(None);
        }

        if let Some(post) = self.post_cache.get(url).await? {
            return Ok(Some(post));
        }

        let post = with_connection!(self.db_pool, |db_conn| {
            posts::table
                .filter(posts::url.eq(url))
                .select(Post::as_select())
                .first(db_conn)
                .await
                .optional()
        })?;

        if let Some(post) = post {
            self.post_cache.set(url, &post).await?;
            return Ok(Some(post));
        }

        let Some(object) = self.fetch_ap_resource(url).await? else {
            return Ok(None);
        };

        let process_data = ProcessNewObject::builder()
            .call_depth(call_depth)
            .db_pool(&self.db_pool)
            .embed_client(self.embed_client.as_ref())
            .fetcher(self)
            .language_detection_config(self.language_detection_config)
            .object(Box::new(object))
            .search_backend(&self.search_backend)
            .build();
        let post = process_new_object(process_data).await?;

        self.post_cache.set(&post.url, &post).await?;

        Ok(Some(post))
    }
}
