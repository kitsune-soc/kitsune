use crate::m20220101_000001_create_table::{Accounts, Posts};
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum MediaAttachments {
    Table,
    Id,
    AccountId,
    ContentType,
    Description,
    Blurhash,
    FilePath,
    RemoteUrl,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum PostsMediaAttachments {
    Table,
    PostId,
    MediaAttachmentId,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MediaAttachments::Table)
                    .col(ColumnDef::new(MediaAttachments::Id).uuid().primary_key())
                    .col(
                        ColumnDef::new(MediaAttachments::AccountId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MediaAttachments::ContentType)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(MediaAttachments::Description).text())
                    .col(ColumnDef::new(MediaAttachments::Blurhash).text())
                    .col(ColumnDef::new(MediaAttachments::FilePath).text())
                    .col(ColumnDef::new(MediaAttachments::RemoteUrl).text())
                    .col(
                        ColumnDef::new(MediaAttachments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MediaAttachments::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(MediaAttachments::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PostsMediaAttachments::Table)
                    .col(
                        ColumnDef::new(PostsMediaAttachments::PostId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PostsMediaAttachments::MediaAttachmentId)
                            .uuid()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(PostsMediaAttachments::PostId)
                            .col(PostsMediaAttachments::MediaAttachmentId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PostsMediaAttachments::Table, PostsMediaAttachments::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                PostsMediaAttachments::Table,
                                PostsMediaAttachments::MediaAttachmentId,
                            )
                            .to(MediaAttachments::Table, MediaAttachments::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Accounts::Table)
                    .add_column_if_not_exists(ColumnDef::new(Accounts::AvatarId).uuid())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_col(Accounts::AvatarId)
                            .to_tbl(MediaAttachments::Table)
                            .to_col(MediaAttachments::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Accounts::Table)
                    .add_column_if_not_exists(ColumnDef::new(Accounts::HeaderId).uuid())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_col(Accounts::HeaderId)
                            .to_tbl(MediaAttachments::Table)
                            .to_col(MediaAttachments::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Accounts::Table)
                    .drop_column(Accounts::AvatarId)
                    .drop_column(Accounts::HeaderId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(PostsMediaAttachments::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(MediaAttachments::Table).to_owned())
            .await
    }
}
