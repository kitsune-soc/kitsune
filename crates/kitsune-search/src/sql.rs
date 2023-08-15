use super::{Result, SearchBackend, SearchIndex, SearchItem, SearchResult};
use async_trait::async_trait;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use diesel_full_text_search::{websearch_to_tsquery_with_search_config, TsVectorExtensions};
use futures_util::TryStreamExt;
use kitsune_db::{
    function::iso_code_to_language,
    lang::LanguageIsoCode,
    model::post::Visibility,
    schema::{accounts, posts},
    PgPool,
};
use speedy_uuid::Uuid;

#[derive(Clone)]
pub struct SearchService {
    db_conn: PgPool,
}

impl SearchService {
    #[must_use]
    pub fn new(db_conn: PgPool) -> Self {
        Self { db_conn }
    }
}

#[async_trait]
impl SearchBackend for SearchService {
    async fn add_to_index(&self, _item: SearchItem) -> Result<()> {
        Ok(())
    }

    async fn remove_from_index(&self, _item: SearchItem) -> Result<()> {
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
        let mut db_conn = self.db_conn.get().await?;

        let query_lang = kitsune_lang_id::get_iso_code(&query);
        let query_fn_call = websearch_to_tsquery_with_search_config(
            iso_code_to_language(LanguageIsoCode::from(query_lang)),
            &query,
        );

        match index {
            SearchIndex::Account => {
                let mut query = accounts::table
                    .filter(
                        accounts::display_name_ts
                            .matches(&query_fn_call)
                            .or(accounts::note_ts
                                .matches(&query_fn_call)
                                .or(accounts::username_ts.matches(&query_fn_call))),
                    )
                    .into_boxed();

                if let Some(min_id) = min_id {
                    query = query.filter(accounts::id.gt(min_id));
                }
                if let Some(max_id) = max_id {
                    query = query.filter(accounts::id.lt(max_id));
                }

                let results = query
                    .limit(max_results as i64)
                    .offset(offset as i64)
                    .select(accounts::id)
                    .load_stream(&mut db_conn)
                    .await?
                    .map_ok(|id| SearchResult { id })
                    .try_collect()
                    .await?;

                Ok(results)
            }
            SearchIndex::Post => {
                let mut query = posts::table
                    .filter(
                        posts::content_ts
                            .matches(&query_fn_call)
                            .or(posts::subject_ts.matches(&query_fn_call)),
                    )
                    .into_boxed();

                if let Some(min_id) = min_id {
                    query = query.filter(posts::id.gt(min_id));
                }
                if let Some(max_id) = max_id {
                    query = query.filter(posts::id.lt(max_id));
                }

                let results = query
                    .filter(posts::visibility.eq_any([Visibility::Public, Visibility::Unlisted]))
                    .limit(max_results as i64)
                    .offset(offset as i64)
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
