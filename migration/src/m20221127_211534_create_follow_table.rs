use crate::m20220101_000001_create_table::Accounts;
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum AccountsFollowers {
    Table,
    Id,
    AccountId,
    FollowerId,
    ApprovedAt,
    Url,
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
                    .table(AccountsFollowers::Table)
                    .col(ColumnDef::new(AccountsFollowers::Id).uuid().primary_key())
                    .col(
                        ColumnDef::new(AccountsFollowers::AccountId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AccountsFollowers::FollowerId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AccountsFollowers::ApprovedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(AccountsFollowers::Url)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(AccountsFollowers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AccountsFollowers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .col(AccountsFollowers::AccountId)
                            .col(AccountsFollowers::FollowerId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(AccountsFollowers::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(AccountsFollowers::FollowerId)
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
            .drop_table(Table::drop().table(AccountsFollowers::Table).to_owned())
            .await
    }
}
