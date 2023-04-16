use crate::m20220101_000001_create_table::Accounts;
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum AccountsBlocks {
    Table,
    Id,
    AccountId,
    BlockedAccountId,
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
                    .table(AccountsBlocks::Table)
                    .col(ColumnDef::new(AccountsBlocks::Id).uuid().primary_key())
                    .col(ColumnDef::new(AccountsBlocks::AccountId).uuid().not_null())
                    .col(
                        ColumnDef::new(AccountsBlocks::BlockedAccountId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AccountsBlocks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .col(AccountsBlocks::AccountId)
                            .col(AccountsBlocks::BlockedAccountId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(AccountsBlocks::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(AccountsBlocks::BlockedAccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AccountsBlocks::Table).to_owned())
            .await
    }
}
