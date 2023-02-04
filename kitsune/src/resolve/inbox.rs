use crate::error::Result;
use futures_util::{future::Either, Stream, StreamExt};
use kitsune_db::{
    column::InboxUrlQuery,
    custom::Visibility,
    entity::{accounts, posts, prelude::Accounts},
    link::{Followers, MentionedAccounts},
};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, ModelTrait, QuerySelect};

pub struct InboxResolver {
    db_conn: DatabaseConnection,
}

impl InboxResolver {
    #[must_use]
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }

    #[instrument(skip_all, fields(post_id = %post.id))]
    pub async fn resolve(
        &self,
        post: &posts::Model,
    ) -> Result<impl Stream<Item = Result<String, DbErr>> + Send + '_> {
        let account = Accounts::find_by_id(post.account_id)
            .one(&self.db_conn)
            .await?
            .expect("[Bug] Post without associated account");

        let mentioned_inbox_stream = post
            .find_linked(MentionedAccounts)
            .select_only()
            .column(accounts::Column::InboxUrl)
            .into_values::<String, InboxUrlQuery>()
            .stream(&self.db_conn)
            .await?;

        Ok(if post.visibility == Visibility::MentionOnly {
            Either::Left(mentioned_inbox_stream)
        } else {
            let follower_inbox_stream = account
                .find_linked(Followers)
                .select_only()
                .column(accounts::Column::InboxUrl)
                .into_values::<_, InboxUrlQuery>()
                .stream(&self.db_conn)
                .await?;

            Either::Right(mentioned_inbox_stream.chain(follower_inbox_stream))
        })
    }
}
