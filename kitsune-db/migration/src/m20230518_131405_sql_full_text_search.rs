use crate::m20220101_000001_create_table::{Accounts, Posts};
use kitsune_db_common::{
    to_tsvector, tsvector_column,
    types::{PgCompositeIndex, PgTypes},
    StoredGeneratedColumn,
};
use sea_orm_migration::{
    prelude::*,
    sea_orm::{DatabaseBackend, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        match manager.get_database_backend() {
            DatabaseBackend::Postgres => {
                // Add a generated tsvector column to the database and create a GIN index over it for fast full-text search

                // --- START POSTS ---
                manager
                    .alter_table(
                        Table::alter()
                            .table(Posts::Table)
                            .add_column(ColumnDef::new(tsvector_column::Posts::Content).custom(
                                StoredGeneratedColumn::new(PgTypes::Tsvector).generate_expr(
                                    to_tsvector("simple", Expr::col(Posts::Content)),
                                ),
                            ))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .create_index(
                        Index::create()
                            .name("idx-posts-content-ts")
                            .table(Posts::Table)
                            .col(tsvector_column::Posts::Content)
                            .index_type(IndexType::Custom(PgCompositeIndex::Gin.into_iden()))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(Posts::Table)
                            .add_column(ColumnDef::new(tsvector_column::Posts::Subject).custom(
                                StoredGeneratedColumn::new(PgTypes::Tsvector).generate_expr(
                                    to_tsvector("simple", Expr::col(Posts::Subject)),
                                ),
                            ))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .create_index(
                        Index::create()
                            .name("idx-posts-subject-ts")
                            .table(Posts::Table)
                            .col(tsvector_column::Posts::Subject)
                            .index_type(IndexType::Custom(PgCompositeIndex::Gin.into_iden()))
                            .to_owned(),
                    )
                    .await?;
                // --- END POSTS ---

                // --- START ACCOUNTS ---
                manager
                    .alter_table(
                        Table::alter()
                            .table(Accounts::Table)
                            .add_column(
                                ColumnDef::new(tsvector_column::Accounts::DisplayName).custom(
                                    StoredGeneratedColumn::new(PgTypes::Tsvector).generate_expr(
                                        to_tsvector("simple", Expr::col(Accounts::DisplayName)),
                                    ),
                                ),
                            )
                            .to_owned(),
                    )
                    .await?;

                manager
                    .create_index(
                        Index::create()
                            .name("idx-accounts-display_name-ts")
                            .table(Accounts::Table)
                            .col(tsvector_column::Accounts::DisplayName)
                            .index_type(IndexType::Custom(PgCompositeIndex::Gin.into_iden()))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(Accounts::Table)
                            .add_column(ColumnDef::new(tsvector_column::Accounts::Note).custom(
                                StoredGeneratedColumn::new(PgTypes::Tsvector).generate_expr(
                                    to_tsvector("simple", Expr::col(Accounts::Note)),
                                ),
                            ))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .create_index(
                        Index::create()
                            .name("idx-accounts-note-ts")
                            .table(Accounts::Table)
                            .col(tsvector_column::Accounts::Note)
                            .index_type(IndexType::Custom(PgCompositeIndex::Gin.into_iden()))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(Accounts::Table)
                            .add_column(ColumnDef::new(tsvector_column::Accounts::Username).custom(
                                StoredGeneratedColumn::new(PgTypes::Tsvector).generate_expr(
                                    to_tsvector("simple", Expr::col(Accounts::Username)),
                                ),
                            ))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .create_index(
                        Index::create()
                            .name("idx-accounts-username-ts")
                            .table(Accounts::Table)
                            .col(tsvector_column::Accounts::Username)
                            .index_type(IndexType::Custom(PgCompositeIndex::Gin.into_iden()))
                            .to_owned(),
                    )
                    .await?;
                // --- END ACCOUNTS ---
            }
            DatabaseBackend::Sqlite => {
                // Create a new FTS5 virtual table and some triggers to automatically maintain its dataset

                // --- START POSTS ---
                let statement = Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    r#"
                        CREATE VIRTUAL TABLE posts_fts USING fts5 (
                            id UNINDEXED,
                            subject,
                            content
                        );

                        CREATE TRIGGER posts_fts_ai AFTER INSERT ON posts BEGIN
                            INSERT INTO posts_fts(id, subject, content) VALUES (new.id, new.subject, new.content);
                        END;

                        CREATE TRIGGER posts_fts_ad AFTER DELETE ON posts BEGIN
                            DELETE FROM posts_fts WHERE id = old.id;
                        END;

                        CREATE TRIGGER posts_fts_au AFTER UPDATE ON posts BEGIN
                            DELETE FROM posts_fts WHERE id = old.id;
                            INSERT INTO posts_fts(id, subject, content) VALUES (new.id, new.subject, new.content);
                        END;
                    "#,
                    [],
                );

                manager.get_connection().execute(statement).await?;
                // --- END POSTS ---

                // --- START ACCOUNTS ---
                let statement = Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    r#"
                        CREATE VIRTUAL TABLE accounts_fts USING fts5 (
                            id UNINDEXED,
                            display_name,
                            note,
                            username
                        );

                        CREATE TRIGGER accounts_fts_ai AFTER INSERT ON accounts BEGIN
                            INSERT INTO accounts_fts(id, display_name, note, username) VALUES (new.id, new.display_name, new.note, new.username);
                        END;

                        CREATE TRIGGER accounts_fts_ad AFTER DELETE ON accounts BEGIN
                            DELETE FROM accounts_fts WHERE id = old.id;
                        END;

                        CREATE TRIGGER accounts_fts_au AFTER UPDATE ON accounts BEGIN
                            DELETE FROM accounts_fts WHERE id = old.id;
                            INSERT INTO accounts_fts(id, display_name, note, username) VALUES (new.id, new.display_name, new.note, new.username);
                        END;
                    "#,
                    [],
                );

                manager.get_connection().execute(statement).await?;
                // --- END ACCOUNTS ---
            }
            DatabaseBackend::MySql => panic!("Unsupported backend"),
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        match manager.get_database_backend() {
            DatabaseBackend::Postgres => {
                manager
                    .drop_index(Index::drop().name("idx-accounts-username-ts").to_owned())
                    .await?;

                manager
                    .drop_index(Index::drop().name("idx-accounts-note-ts").to_owned())
                    .await?;

                manager
                    .drop_index(
                        Index::drop()
                            .name("idx-accounts-display_name-ts")
                            .to_owned(),
                    )
                    .await?;

                manager
                    .drop_index(Index::drop().name("idx-posts-subject-ts").to_owned())
                    .await?;

                manager
                    .drop_index(Index::drop().name("idx-posts-content-ts").to_owned())
                    .await?;
            }
            DatabaseBackend::Sqlite => {
                let statement = Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    r#"
                        DROP TRIGGER posts_fts_au;
                        DROP TRIGGER posts_fts_ad;
                        DROP TRIGGER posts_fts_ai;
                    "#,
                    [],
                );
                manager.get_connection().execute(statement).await?;

                manager
                    .drop_table(Table::drop().table(Alias::new("posts_fts")).to_owned())
                    .await?;

                let statement = Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    r#"
                        DROP TRIGGER accounts_fts_au;
                        DROP TRIGGER accounts_fts_ad;
                        DROP TRIGGER accounts_fts_ai;
                    "#,
                    [],
                );
                manager.get_connection().execute(statement).await?;

                manager
                    .drop_table(Table::drop().table(Alias::new("accounts_fts")).to_owned())
                    .await?;
            }
            DatabaseBackend::MySql => panic!("Unsupported database backend"),
        }

        Ok(())
    }
}
