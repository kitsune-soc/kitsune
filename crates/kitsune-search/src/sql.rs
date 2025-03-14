use super::{Result, SearchBackend, SearchIndex, SearchItem, SearchResultReference};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use diesel_full_text_search::{TsVectorExtensions, websearch_to_tsquery_with_search_config};
use futures_util::TryStreamExt;
use kitsune_config::language_detection::Configuration as LanguageDetectionConfig;
use kitsune_db::{
    PgPool,
    function::iso_code_to_language,
    lang::LanguageIsoCode,
    model::post::Visibility,
    schema::{accounts, posts},
    with_connection,
};
use kitsune_derive::kitsune_service;
use speedy_uuid::Uuid;

#[kitsune_service]
pub struct SearchService {
    db_pool: PgPool,
    language_detection_config: LanguageDetectionConfig,
}

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

    #[cfg_attr(not(coverage), instrument(skip(self)))]
    async fn search(
        &self,
        index: SearchIndex,
        query: &str,
        max_results: u64,
        offset: u64,
        min_id: Option<Uuid>,
        max_id: Option<Uuid>,
    ) -> Result<Vec<SearchResultReference>> {
        let query_lang = kitsune_language::detect_language(self.language_detection_config, query);
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

                let results = with_connection!(self.db_pool, |db_conn| {
                    query
                        .limit(max_results as i64)
                        .offset(offset as i64)
                        .select(accounts::id)
                        .load_stream(db_conn)
                        .await?
                        .map_ok(|id| SearchResultReference { index, id })
                        .try_collect()
                        .await
                })?;

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

                let results = with_connection!(self.db_pool, |db_conn| {
                    query
                        .filter(
                            posts::visibility.eq_any([Visibility::Public, Visibility::Unlisted]),
                        )
                        .limit(max_results as i64)
                        .offset(offset as i64)
                        .select(posts::id)
                        .load_stream(db_conn)
                        .await?
                        .map_ok(|id| SearchResultReference { index, id })
                        .try_collect()
                        .await
                })?;

                Ok(results)
            }
        }
    }

    async fn update_in_index(&self, _item: SearchItem) -> Result<()> {
        Ok(())
    }
}
