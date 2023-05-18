use sea_orm_migration::{
    prelude::*,
    sea_orm::{DatabaseBackend, Statement},
};

use crate::m20220101_000001_create_table::{Accounts, Posts};

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
                            .add_column(ColumnDef::new(Posts::ContentTsvector).custom(Alias::new(
                                "tsvector GENERATED ALWAYS AS (to_tsvector('simple', content)) STORED",
                            )))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .create_index(
                        Index::create()
                            .name("idx-posts-content-ts")
                            .table(Posts::Table)
                            .col(Posts::ContentTsvector)
                            .index_type(IndexType::Custom(SeaRc::new(Alias::new("GIN"))))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(Posts::Table)
                            .add_column(ColumnDef::new(Posts::SubjectTsvector).custom(Alias::new(
                                "tsvector GENERATED ALWAYS AS (to_tsvector('simple', content)) STORED",
                            )))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .create_index(
                        Index::create()
                            .name("idx-posts-subject-ts")
                            .table(Posts::Table)
                            .col(Posts::SubjectTsvector)
                            .index_type(IndexType::Custom(SeaRc::new(Alias::new("GIN"))))
                            .to_owned(),
                    )
                    .await?;
                // --- END POSTS ---

                // --- START ACCOUNTS ---
                manager
                    .alter_table(
                        Table::alter()
                            .table(Accounts::Table)
                            .add_column(ColumnDef::new(Accounts::DisplayNameTsvector).custom(Alias::new(
                                "tsvector GENERATED ALWAYS AS (to_tsvector('simple', display_name)) STORED",
                            )))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .create_index(
                        Index::create()
                            .name("idx-accounts-display_name-ts")
                            .table(Accounts::Table)
                            .col(Accounts::DisplayNameTsvector)
                            .index_type(IndexType::Custom(SeaRc::new(Alias::new("GIN"))))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(Accounts::Table)
                            .add_column(ColumnDef::new(Accounts::NoteTsvector).custom(Alias::new(
                                "tsvector GENERATED ALWAYS AS (to_tsvector('simple', note)) STORED",
                            )))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .create_index(
                        Index::create()
                            .name("idx-accounts-note-ts")
                            .table(Accounts::Table)
                            .col(Accounts::NoteTsvector)
                            .index_type(IndexType::Custom(SeaRc::new(Alias::new("GIN"))))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(Accounts::Table)
                            .add_column(ColumnDef::new(Accounts::UsernameTsvector).custom(Alias::new(
                                "tsvector GENERATED ALWAYS AS (to_tsvector('simple', username)) STORED",
                            )))
                            .to_owned(),
                    )
                    .await?;

                manager
                    .create_index(
                        Index::create()
                            .name("idx-accounts-username-ts")
                            .table(Accounts::Table)
                            .col(Accounts::UsernameTsvector)
                            .index_type(IndexType::Custom(SeaRc::new(Alias::new("GIN"))))
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
