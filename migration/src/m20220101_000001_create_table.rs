use sea_orm_migration::prelude::*;

#[derive(Iden)]
enum Posts {
    Table,
    Id,
    UserId,
    Subject,
    Content,
    Url,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    Avatar,
    Header,
    DisplayName,
    Note,
    Username,
    Email,
    Password,
    Domain,
    Url,
    InboxUrl,
    PublicKey,
    PrivateKey,
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
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().primary_key())
                    .col(ColumnDef::new(Users::Avatar).text())
                    .col(ColumnDef::new(Users::Header).text())
                    .col(ColumnDef::new(Users::DisplayName).text())
                    .col(ColumnDef::new(Users::Note).text())
                    .col(ColumnDef::new(Users::Username).text().not_null())
                    .col(ColumnDef::new(Users::Email).text())
                    .col(ColumnDef::new(Users::Password).text())
                    .col(ColumnDef::new(Users::Domain).text())
                    .col(ColumnDef::new(Users::Url).text().not_null().unique_key())
                    .col(ColumnDef::new(Users::InboxUrl).text().not_null())
                    .col(ColumnDef::new(Users::PublicKey).text())
                    .col(ColumnDef::new(Users::PrivateKey).text())
                    // TODO: Figure out triggers for created at and updated at
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .col(Users::Username)
                            .col(Users::Domain)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Posts::Id).uuid().primary_key())
                    .col(ColumnDef::new(Posts::UserId).uuid().not_null())
                    .col(ColumnDef::new(Posts::Subject).text())
                    .col(ColumnDef::new(Posts::Content).text().not_null())
                    .col(ColumnDef::new(Posts::Url).text().not_null().unique_key())
                    // TODO: Figure out triggers for created at and updated at
                    .col(
                        ColumnDef::new(Posts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Posts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::UserId)
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
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}
