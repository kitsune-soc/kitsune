use crate::m20220101_000001_create_table::{Posts, Users};
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Reposts {
    Table,
    UserId,
    PostId,
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
                    .if_not_exists()
                    .table(Reposts::Table)
                    .col(ColumnDef::new(Reposts::UserId).uuid().not_null())
                    .col(ColumnDef::new(Reposts::PostId).uuid().not_null())
                    .col(ColumnDef::new(Reposts::Url).text().not_null().unique_key())
                    .col(
                        ColumnDef::new(Reposts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .primary_key(Index::create().col(Reposts::UserId).col(Reposts::PostId))
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Reposts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Reposts::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Reposts::Table).to_owned())
            .await
    }
}
