use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Accounts {
    Table,
    Id,
    AvatarId,
    HeaderId,
    DisplayName,
    Note,
    Username,
    Locked,
    Local,
    Domain,
    Url,
    FollowersUrl,
    InboxUrl,
    PublicKey,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum Posts {
    Table,
    Id,
    AccountId,
    InReplyToId,
    RepostedPostId,
    IsSensitive,
    Subject,
    Content,
    Visibility,
    IsLocal,
    Url,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    AccountId,
    OidcId,
    Username,
    Email,
    Password,
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
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Accounts::Id).uuid().primary_key())
                    .col(ColumnDef::new(Accounts::DisplayName).text())
                    .col(ColumnDef::new(Accounts::Note).text())
                    .col(ColumnDef::new(Accounts::Username).text().not_null())
                    .col(ColumnDef::new(Accounts::Locked).boolean().not_null())
                    .col(ColumnDef::new(Accounts::Local).boolean().not_null())
                    .col(ColumnDef::new(Accounts::Domain).text())
                    .col(ColumnDef::new(Accounts::Url).text().not_null().unique_key())
                    .col(ColumnDef::new(Accounts::FollowersUrl).text().not_null())
                    .col(ColumnDef::new(Accounts::InboxUrl).text().not_null())
                    .col(ColumnDef::new(Accounts::PublicKey).text().not_null())
                    .col(
                        ColumnDef::new(Accounts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Accounts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .col(Accounts::Username)
                            .col(Accounts::Domain)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().primary_key())
                    .col(
                        ColumnDef::new(Users::AccountId)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::OidcId).text().unique_key())
                    .col(
                        ColumnDef::new(Users::Username)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::Email).text().not_null().unique_key())
                    .col(ColumnDef::new(Users::Password).text().unique_key())
                    .col(ColumnDef::new(Users::PrivateKey).text().not_null())
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
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Users::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
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
                    .col(ColumnDef::new(Posts::AccountId).uuid().not_null())
                    .col(ColumnDef::new(Posts::InReplyToId).uuid())
                    .col(ColumnDef::new(Posts::RepostedPostId).uuid())
                    .col(ColumnDef::new(Posts::IsSensitive).boolean().not_null())
                    .col(ColumnDef::new(Posts::Subject).text())
                    .col(ColumnDef::new(Posts::Content).text().not_null())
                    .col(ColumnDef::new(Posts::Visibility).integer().not_null())
                    .col(ColumnDef::new(Posts::IsLocal).boolean().not_null())
                    .col(ColumnDef::new(Posts::Url).text().not_null().unique_key())
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
                            .from(Posts::Table, Posts::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::InReplyToId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::RepostedPostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-posts-account-id")
                    .table(Posts::Table)
                    .col(Posts::AccountId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-posts-account-id")
                    .table(Posts::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await
    }
}
