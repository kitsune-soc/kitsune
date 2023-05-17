use sea_orm_migration::{
    prelude::*,
    sea_orm::{DatabaseBackend, Statement},
};

#[derive(Iden)]
pub enum Accounts {
    Table,
    Id,
    AvatarId,
    HeaderId,
    DisplayName,
    Note,
    Username,
    Locked,
    Local,
    Domain,

    // ActivityPub data
    ActorType,
    Url,
    FeaturedCollectionUrl,
    FollowersUrl,
    FollowingUrl,
    InboxUrl,
    OutboxUrl,
    SharedInboxUrl,

    // HTTP signature data
    PublicKeyId,
    PublicKey,

    // Full-text search vectors (PostgreSQL only)
    DisplayNameTsvector,
    NoteTsvector,
    UsernameTsvector,

    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum Posts {
    Table,
    Id,
    AccountId,
    InReplyToId,
    RepostedPostId,
    IsSensitive,
    Subject,
    Content,
    Visibility,
    IsLocal,
    Url,
    CreatedAt,
    UpdatedAt,

    // Full-text search vectors (PostgreSQL only)
    SubjectTsvector,
    ContentTsvector,
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    AccountId,
    OidcId,
    Username,
    Email,
    Password,
    Domain,
    PrivateKey,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Accounts::Id).uuid().primary_key())
                    .col(ColumnDef::new(Accounts::DisplayName).text())
                    .col(ColumnDef::new(Accounts::Note).text())
                    .col(ColumnDef::new(Accounts::Username).text().not_null())
                    .col(ColumnDef::new(Accounts::Locked).boolean().not_null())
                    .col(ColumnDef::new(Accounts::Local).boolean().not_null())
                    .col(ColumnDef::new(Accounts::Domain).text().not_null())
                    .col(ColumnDef::new(Accounts::ActorType).integer().not_null())
                    .col(ColumnDef::new(Accounts::Url).text().unique_key())
                    .col(ColumnDef::new(Accounts::FeaturedCollectionUrl).text())
                    .col(ColumnDef::new(Accounts::FollowersUrl).text())
                    .col(ColumnDef::new(Accounts::FollowingUrl).text())
                    .col(ColumnDef::new(Accounts::InboxUrl).text())
                    .col(ColumnDef::new(Accounts::OutboxUrl).text())
                    .col(ColumnDef::new(Accounts::SharedInboxUrl).text())
                    .col(
                        ColumnDef::new(Accounts::PublicKeyId)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Accounts::PublicKey).text().not_null())
                    .col(
                        ColumnDef::new(Accounts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Accounts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .col(Accounts::Username)
                            .col(Accounts::Domain)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().primary_key())
                    .col(
                        ColumnDef::new(Users::AccountId)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::OidcId).text().unique_key())
                    .col(ColumnDef::new(Users::Username).text().not_null())
                    .col(ColumnDef::new(Users::Email).text().not_null().unique_key())
                    .col(ColumnDef::new(Users::Password).text().unique_key())
                    .col(ColumnDef::new(Users::Domain).text().not_null())
                    .col(ColumnDef::new(Users::PrivateKey).text().not_null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Users::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .col(Users::Username)
                            .col(Users::Domain)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Posts::Id).uuid().primary_key())
                    .col(ColumnDef::new(Posts::AccountId).uuid().not_null())
                    .col(ColumnDef::new(Posts::InReplyToId).uuid())
                    .col(ColumnDef::new(Posts::RepostedPostId).uuid())
                    .col(ColumnDef::new(Posts::IsSensitive).boolean().not_null())
                    .col(ColumnDef::new(Posts::Subject).text())
                    .col(ColumnDef::new(Posts::Content).text().not_null())
                    .col(ColumnDef::new(Posts::Visibility).integer().not_null())
                    .col(ColumnDef::new(Posts::IsLocal).boolean().not_null())
                    .col(ColumnDef::new(Posts::Url).text().not_null().unique_key())
                    .col(
                        ColumnDef::new(Posts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Posts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::InReplyToId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::RepostedPostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-posts-account_id")
                    .table(Posts::Table)
                    .col(Posts::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-posts-reposted_post_id")
                    .table(Posts::Table)
                    .col(Posts::RepostedPostId)
                    .to_owned(),
            )
            .await?;

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

        manager
            .drop_index(Index::drop().name("idx-posts-reposted_post_id").to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-posts-account_id")
                    .table(Posts::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await
    }
}
