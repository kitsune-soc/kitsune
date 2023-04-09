use crate::m20220101_000001_create_table::{Accounts, Posts};
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum PostsFavourites {
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
                    .table(PostsFavourites::Table)
                    .col(ColumnDef::new(PostsFavourites::Id).uuid().primary_key())
                    .col(ColumnDef::new(PostsFavourites::AccountId).uuid().not_null())
                    .col(ColumnDef::new(PostsFavourites::PostId).uuid().not_null())
                    .col(
                        ColumnDef::new(PostsFavourites::Url)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(PostsFavourites::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .col(PostsFavourites::AccountId)
                            .col(PostsFavourites::PostId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(PostsFavourites::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(PostsFavourites::PostId)
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
            .drop_table(Table::drop().table(PostsFavourites::Table).to_owned())
            .await
    }
}
