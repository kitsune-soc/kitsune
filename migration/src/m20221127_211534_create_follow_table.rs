use crate::m20220101_000001_create_table::Users;
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum UsersFollowers {
    Table,
    UserId,
    FollowerId,
    ApprovedAt,
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
                    .table(UsersFollowers::Table)
                    .col(ColumnDef::new(UsersFollowers::UserId).uuid().not_null())
                    .col(ColumnDef::new(UsersFollowers::FollowerId).uuid().not_null())
                    .col(
                        ColumnDef::new(UsersFollowers::ApprovedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UsersFollowers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UsersFollowers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(UsersFollowers::UserId)
                            .col(UsersFollowers::FollowerId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(UsersFollowers::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(UsersFollowers::FollowerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UsersFollowers::Table).to_owned())
            .await
    }
}
