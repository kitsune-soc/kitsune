use sea_orm_migration::prelude::*;

#[derive(Iden)]
enum Jobs {
    Table,
    Id,
    State,
    Context,
    RunAt,
    FailCount,
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
                    .table(Jobs::Table)
                    .col(ColumnDef::new(Jobs::Id).uuid().primary_key())
                    .col(ColumnDef::new(Jobs::State).big_unsigned().not_null())
                    .col(ColumnDef::new(Jobs::Context).json_binary().not_null())
                    .col(
                        ColumnDef::new(Jobs::RunAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Jobs::FailCount)
                            .big_unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Jobs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Jobs::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Jobs::Table).to_owned())
            .await
    }
}
