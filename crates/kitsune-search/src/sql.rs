use super::{Result, SearchBackend, SearchIndex, SearchItem, SearchResult};
use async_trait::async_trait;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
use diesel_full_text_search::{websearch_to_tsquery_with_search_config, TsVectorExtensions};
use futures_util::TryStreamExt;
use kitsune_db::{
    function::iso_code_to_language,
    lang::LanguageIsoCode,
    model::post::Visibility,
    schema::{accounts, posts},
    PgPool,
};
use kitsune_language::DetectionBackend;
use speedy_uuid::Uuid;

#[derive(Clone)]
pub struct SearchService {
    db_pool: PgPool,
}

impl SearchService {
    #[must_use]
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl SearchBackend for SearchService {
    async fn add_to_index(&self, _item: SearchItem) -> Result<()> {
        Ok(())
    }

    async fn remove_from_index(&self, _item: &SearchItem) -> Result<()> {
        Ok(())
    }

    async fn reset_index(&self, _index: SearchIndex) -> Result<()> {
        Ok(())
    }

    #[instrument(skip(self))]
    async fn search(
        &self,
        index: SearchIndex,
        query: String,
        max_results: u64,
        offset: u64,
        min_id: Option<Uuid>,
        max_id: Option<Uuid>,
    ) -> Result<Vec<SearchResult>> {
        let query_lang = kitsune_language::detect_language(DetectionBackend::default(), &query);
        let query_fn_call = websearch_to_tsquery_with_search_config(
            iso_code_to_language(LanguageIsoCode::from(query_lang)),
            &query,
        );

        match index {
            SearchIndex::Account => {
                let mut query = accounts::table
                    .filter(accounts::account_ts.matches(&query_fn_call))
                    .into_boxed();

                if let Some(min_id) = min_id {
                    query = query.filter(accounts::id.gt(min_id));
                }
                if let Some(max_id) = max_id {
                    query = query.filter(accounts::id.lt(max_id));
                }

                let results = self
                    .db_pool
                    .with_connection(move |db_conn| {
                        async move {
                            query
                                .limit(max_results as i64)
                                .offset(offset as i64)
                                .select(accounts::id)
                                .load_stream(db_conn)
                                .await?
                                .map_ok(|id| SearchResult { id })
                                .try_collect()
                                .await
                        }
                        .scoped()
                    })
                    .await?;

                Ok(results)
            }
            SearchIndex::Post => {
                let mut query = posts::table
                    .filter(posts::post_ts.matches(&query_fn_call))
                    .into_boxed();

                if let Some(min_id) = min_id {
                    query = query.filter(posts::id.gt(min_id));
                }
                if let Some(max_id) = max_id {
                    query = query.filter(posts::id.lt(max_id));
                }

                let results = self
                    .db_pool
                    .with_connection(|db_conn| {
                        async move {
                            query
                                .filter(
                                    posts::visibility
                                        .eq_any([Visibility::Public, Visibility::Unlisted]),
                                )
                                .limit(max_results as i64)
                                .offset(offset as i64)
                                .select(posts::id)
                                .load_stream(db_conn)
                                .await?
                                .map_ok(|id| SearchResult { id })
                                .try_collect()
                                .await
                        }
                        .scoped()
                    })
                    .await?;

                Ok(results)
            }
        }
    }

    async fn update_in_index(&self, _item: SearchItem) -> Result<()> {
        Ok(())
    }
}
