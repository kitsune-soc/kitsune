use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_table::Users;

#[derive(Iden)]
enum OAuth2AccessTokens {
    Table,
    Token,
    UserId,
    ApplicationId,
    CreatedAt,
    ExpiredAt,
}

#[derive(Iden)]
enum OAuth2Applications {
    Table,
    Id,
    Name,
    Secret,
    RedirectUri,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum OAuth2RefreshTokens {
    Table,
    Token,
    AccessToken,
    ApplicationId,
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
                    .table(OAuth2Applications::Table)
                    .col(ColumnDef::new(OAuth2Applications::Id).uuid().primary_key())
                    .col(ColumnDef::new(OAuth2Applications::Name).text().not_null())
                    .col(
                        ColumnDef::new(OAuth2Applications::Secret)
                            .text()
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OAuth2Applications::RedirectUri)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OAuth2Applications::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OAuth2Applications::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OAuth2AccessTokens::Table)
                    .col(
                        ColumnDef::new(OAuth2AccessTokens::Token)
                            .text()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OAuth2AccessTokens::UserId).uuid())
                    .col(ColumnDef::new(OAuth2AccessTokens::ApplicationId).uuid())
                    .col(
                        ColumnDef::new(OAuth2AccessTokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OAuth2AccessTokens::ExpiredAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(OAuth2AccessTokens::ApplicationId)
                            .to(OAuth2Applications::Table, OAuth2Applications::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(OAuth2AccessTokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OAuth2RefreshTokens::Table)
                    .col(
                        ColumnDef::new(OAuth2RefreshTokens::Token)
                            .text()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OAuth2RefreshTokens::AccessToken)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(OAuth2RefreshTokens::ApplicationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OAuth2RefreshTokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(OAuth2RefreshTokens::AccessToken)
                            .to(OAuth2AccessTokens::Table, OAuth2AccessTokens::Token)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(OAuth2RefreshTokens::ApplicationId)
                            .to(OAuth2Applications::Table, OAuth2Applications::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OAuth2RefreshTokens::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(OAuth2AccessTokens::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(OAuth2Applications::Table).to_owned())
            .await
    }
}
