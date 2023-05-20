use super::{Result, SearchBackend, SearchIndex, SearchItem, SearchResult};
use async_trait::async_trait;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use futures_util::TryStreamExt;
use kitsune_db::{
    function::websearch_to_tsquery,
    model::post::Visibility,
    schema::{accounts, posts},
    PgPool,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlSearchService {
    db_conn: PgPool,
}

impl SqlSearchService {
    #[must_use]
    pub fn new(db_conn: PgPool) -> Self {
        Self { db_conn }
    }
}

#[async_trait]
impl SearchBackend for SqlSearchService {
    async fn add_to_index(&self, _item: SearchItem) -> Result<()> {
        Ok(())
    }

    async fn remove_from_index(&self, _item: SearchItem) -> Result<()> {
        Ok(())
    }

    async fn reset_index(&self, _index: SearchIndex) -> Result<()> {
        Ok(())
    }

    async fn search(
        &self,
        index: SearchIndex,
        query: String,
        max_results: u64,
        offset: u64,
        min_id: Option<Uuid>,
        max_id: Option<Uuid>,
    ) -> Result<Vec<SearchResult>> {
        let mut db_conn = self.db_conn.get().await?;

        match index {
            SearchIndex::Account => {
                let mut query = accounts::table.filter(
                    accounts::display_name_ts
                        .matches(websearch_to_tsquery(&query))
                        .or(accounts::note_ts
                            .matches(websearch_to_tsquery(&query))
                            .or(accounts::username_ts.matches(websearch_to_tsquery(&query)))),
                );

                if let Some(min_id) = min_id {
                    query = query.filter(accounts::id.gt(min_id));
                }
                if let Some(max_id) = max_id {
                    query = query.filter(accounts::id.lt(max_id));
                }

                let results = query
                    .limit(max_results)
                    .offset(offset)
                    .select(accounts::id)
                    .load_stream(&mut db_conn)
                    .await?
                    .map_ok(|id| SearchResult { id })
                    .try_collect()
                    .await?;

                Ok(results)
            }
            SearchIndex::Post => {
                let mut query = posts::table.filter(
                    posts::content_ts
                        .matches(websearch_to_tsquery(&query))
                        .or(posts::subject.matches(websearch_to_tsquery(&query))),
                );

                if let Some(min_id) = min_id {
                    query = query.filter(posts::id.gt(min_id));
                }
                if let Some(max_id) = max_id {
                    query = query.filter(posts::id.lt(max_id));
                }

                let results = query
                    .filter(posts::visibility.eq_any([Visibility::Public, Visibility::Unlisted]))
                    .limit(max_results)
                    .offset(offset)
                    .select(posts::id)
                    .load_stream(&mut db_conn)
                    .await?
                    .map_ok(|id| SearchResult { id })
                    .try_collect()
                    .await?;

                Ok(results)
            }
        }
    }
}
