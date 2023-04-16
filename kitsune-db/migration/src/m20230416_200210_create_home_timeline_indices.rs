use crate::{
    m20220101_000001_create_table::Posts, m20221127_211534_create_follow_table::AccountsFollowers,
    m20221213_214258_create_mention_table::PostsMentions,
};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx-accounts_followers-follower_id")
                    .table(AccountsFollowers::Table)
                    .col(AccountsFollowers::FollowerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-posts_mentions-account_id")
                    .table(PostsMentions::Table)
                    .col(PostsMentions::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-posts-visibility")
                    .table(Posts::Table)
                    .col(Posts::Visibility)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx-posts-visibility").to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-posts_mentions-account_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-accounts_followers-follower_id")
                    .to_owned(),
            )
            .await
    }
}
