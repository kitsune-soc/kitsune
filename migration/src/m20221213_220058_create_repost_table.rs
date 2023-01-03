use crate::m20220101_000001_create_table::{Accounts, Posts};
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Reposts {
    Table,
    Id,
    AccountId,
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
                    .col(ColumnDef::new(Reposts::Id).uuid().primary_key())
                    .col(ColumnDef::new(Reposts::AccountId).uuid().not_null())
                    .col(ColumnDef::new(Reposts::PostId).uuid().not_null())
                    .col(ColumnDef::new(Reposts::Url).text().not_null().unique_key())
                    .col(
                        ColumnDef::new(Reposts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .col(Reposts::AccountId)
                            .col(Reposts::PostId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Reposts::AccountId)
                            .to(Accounts::Table, Accounts::Id)
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
