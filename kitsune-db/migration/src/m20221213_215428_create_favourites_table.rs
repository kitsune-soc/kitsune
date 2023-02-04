use crate::m20220101_000001_create_table::{Accounts, Posts};
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Favourites {
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
                    .table(Favourites::Table)
                    .col(ColumnDef::new(Favourites::Id).uuid().primary_key())
                    .col(ColumnDef::new(Favourites::AccountId).uuid().not_null())
                    .col(ColumnDef::new(Favourites::PostId).uuid().not_null())
                    .col(
                        ColumnDef::new(Favourites::Url)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Favourites::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .col(Favourites::AccountId)
                            .col(Favourites::PostId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Favourites::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Favourites::PostId)
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
            .drop_table(Table::drop().table(Favourites::Table).to_owned())
            .await
    }
}
