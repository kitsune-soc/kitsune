use crate::m20220101_000001_create_table::{Accounts, Posts};
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum PostsMentions {
    Table,
    PostId,
    AccountId,
    MentionText,
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
                    .table(PostsMentions::Table)
                    .col(ColumnDef::new(PostsMentions::PostId).uuid().not_null())
                    .col(ColumnDef::new(PostsMentions::AccountId).uuid().not_null())
                    .col(ColumnDef::new(PostsMentions::MentionText).text().not_null())
                    .primary_key(
                        Index::create()
                            .col(PostsMentions::PostId)
                            .col(PostsMentions::AccountId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(PostsMentions::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(PostsMentions::AccountId)
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
            .drop_table(Table::drop().table(PostsMentions::Table).to_owned())
            .await
    }
}
