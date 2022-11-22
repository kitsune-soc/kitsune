use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_table::Users;

#[derive(Iden)]
enum Oauth2AccessTokens {
    Table,
    Token,
    UserId,
    ApplicationId,
    CreatedAt,
    ExpiredAt,
}

#[derive(Iden)]
enum Oauth2Applications {
    Table,
    Id,
    Name,
    Secret,
    RedirectUri,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Oauth2AuthorizationCodes {
    Table,
    Code,
    ApplicationId,
    UserId,
    CreatedAt,
    ExpiredAt,
}

#[derive(Iden)]
enum Oauth2RefreshTokens {
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
                    .table(Oauth2Applications::Table)
                    .col(ColumnDef::new(Oauth2Applications::Id).uuid().primary_key())
                    .col(ColumnDef::new(Oauth2Applications::Name).text().not_null())
                    .col(
                        ColumnDef::new(Oauth2Applications::Secret)
                            .text()
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Oauth2Applications::RedirectUri)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Oauth2Applications::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Oauth2Applications::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Oauth2AuthorizationCodes::Table)
                    .col(
                        ColumnDef::new(Oauth2AuthorizationCodes::Code)
                            .text()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Oauth2AuthorizationCodes::ApplicationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Oauth2AuthorizationCodes::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Oauth2AuthorizationCodes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Oauth2AuthorizationCodes::ExpiredAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Oauth2AuthorizationCodes::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Oauth2AuthorizationCodes::ApplicationId)
                            .to(Oauth2Applications::Table, Oauth2Applications::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Oauth2AccessTokens::Table)
                    .col(
                        ColumnDef::new(Oauth2AccessTokens::Token)
                            .text()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Oauth2AccessTokens::UserId).uuid())
                    .col(ColumnDef::new(Oauth2AccessTokens::ApplicationId).uuid())
                    .col(
                        ColumnDef::new(Oauth2AccessTokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Oauth2AccessTokens::ExpiredAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Oauth2AccessTokens::ApplicationId)
                            .to(Oauth2Applications::Table, Oauth2Applications::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Oauth2AccessTokens::UserId)
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
                    .table(Oauth2RefreshTokens::Table)
                    .col(
                        ColumnDef::new(Oauth2RefreshTokens::Token)
                            .text()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Oauth2RefreshTokens::AccessToken)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Oauth2RefreshTokens::ApplicationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Oauth2RefreshTokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Oauth2RefreshTokens::AccessToken)
                            .to(Oauth2AccessTokens::Table, Oauth2AccessTokens::Token)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Oauth2RefreshTokens::ApplicationId)
                            .to(Oauth2Applications::Table, Oauth2Applications::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Oauth2RefreshTokens::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Oauth2AccessTokens::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Oauth2Applications::Table).to_owned())
            .await
    }
}
