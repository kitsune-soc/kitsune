use crate::m20220101_000001_create_table::Users;
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum UsersRoles {
    Table,
    Id,
    UserId,
    Role,
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
                    .table(UsersRoles::Table)
                    .col(ColumnDef::new(UsersRoles::Id).uuid().primary_key())
                    .col(ColumnDef::new(UsersRoles::UserId).uuid().not_null())
                    .col(ColumnDef::new(UsersRoles::Role).integer().not_null())
                    .col(
                        ColumnDef::new(UsersRoles::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(UsersRoles::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .col(UsersRoles::UserId)
                            .col(UsersRoles::Role)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UsersRoles::Table).to_owned())
            .await
    }
}
