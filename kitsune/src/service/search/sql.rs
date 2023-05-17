use super::{Result, SearchBackend, SearchIndex, SearchItem, SearchResult};
use async_trait::async_trait;
use futures_util::TryStreamExt;
use kitsune_db::{
    custom::Visibility,
    custom_entity::posts_fts,
    entity::{
        accounts, posts,
        prelude::{Accounts, Posts},
    },
};
use sea_orm::{
    sea_query::{Alias, Expr, IntoCondition, PgFunc},
    ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, EntityTrait, JoinType,
    QueryFilter, QuerySelect, RelationTrait,
};
use uuid::Uuid;

#[derive(Clone)]
// TODO: Make case insensitive
pub struct SqlSearchService {
    db_conn: DatabaseConnection,
}

impl SqlSearchService {
    #[must_use]
    pub fn new(db_conn: DatabaseConnection) -> Self {
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
        let like_query = format!("%{query}%");
        match index {
            SearchIndex::Account => {
                let mut query = Accounts::find()
                    .filter(
                        accounts::Column::DisplayName
                            .like(&like_query)
                            .or(accounts::Column::Username.like(&like_query))
                            .or(accounts::Column::Note.like(&like_query)),
                    )
                    .limit(max_results)
                    .offset(offset);

                if let Some(min_id) = min_id {
                    query = query.filter(posts::Column::Id.gt(min_id));
                }
                if let Some(max_id) = max_id {
                    query = query.filter(posts::Column::Id.lt(max_id));
                }

                let results = query
                    .select_only()
                    .column(accounts::Column::Id)
                    .into_tuple()
                    .stream(&self.db_conn)
                    .await?
                    .map_ok(|id| SearchResult { id })
                    .try_collect()
                    .await?;

                Ok(results)
            }
            SearchIndex::Post => {
                let mut query = match self.db_conn.get_database_backend() {
                    DatabaseBackend::Postgres => {
                        use sea_orm::sea_query::extension::postgres::PgExpr;

                        Posts::find().filter(
                            Expr::col(Alias::new("content_tsvector"))
                                .matches(PgFunc::websearch_to_tsquery(&query, None))
                                .or(Expr::col(Alias::new("subject_tsvector"))
                                    .matches(PgFunc::websearch_to_tsquery(&query, None))),
                        )
                    }
                    DatabaseBackend::Sqlite => {
                        use sea_orm::sea_query::extension::sqlite::SqliteExpr;

                        Posts::find().join(
                            JoinType::InnerJoin,
                            posts_fts::Relation::Posts.def().rev().on_condition(
                                move |_posts, posts_fts| {
                                    Expr::col(posts_fts).matches(&query).into_condition()
                                },
                            ),
                        )
                    }
                    DatabaseBackend::MySql => panic!("Unsupported database backend"),
                };

                if let Some(min_id) = min_id {
                    query = query.filter(posts::Column::Id.gt(min_id));
                }
                if let Some(max_id) = max_id {
                    query = query.filter(posts::Column::Id.lt(max_id));
                }

                let results = query
                    .filter(
                        posts::Column::Visibility.is_in([Visibility::Public, Visibility::Unlisted]),
                    )
                    .limit(max_results)
                    .offset(offset)
                    .select_only()
                    .column(posts::Column::Id)
                    .into_tuple()
                    .stream(&self.db_conn)
                    .await?
                    .map_ok(|id| SearchResult { id })
                    .try_collect()
                    .await?;

                Ok(results)
            }
        }
    }
}
