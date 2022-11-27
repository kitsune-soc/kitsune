use crate::m20220101_000001_create_table::Users;
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum MediaAttachments {
    Table,
    Id,
    ContentType,
    Url,
    CreatedAt,
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
                        ColumnDef::new(MediaAttachments::ContentType)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(MediaAttachments::Url).text().not_null())
                    .col(
                        ColumnDef::new(MediaAttachments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column_if_not_exists(ColumnDef::new(Users::AvatarId).uuid())
                    .add_column_if_not_exists(ColumnDef::new(Users::HeaderId).uuid())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_col(Users::AvatarId)
                            .to_tbl(MediaAttachments::Table)
                            .to_col(MediaAttachments::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_col(Users::HeaderId)
                            .to_tbl(MediaAttachments::Table)
                            .to_col(MediaAttachments::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // TODO: Create central ident enum with foreign key names
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::AvatarId)
                    .drop_column(Users::HeaderId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(MediaAttachments::Table).to_owned())
            .await
    }
}
