use crate::{
    db::{
        model::{
            account, follow, mention,
            post::{self, Visibility},
        },
        InboxUrlQuery,
    },
    error::Result,
};
use futures_util::{future::Either, Stream, StreamExt};
use migration::DbErr;
use sea_orm::{DatabaseConnection, ModelTrait, QuerySelect};

pub struct InboxResolver {
    db_conn: DatabaseConnection,
}

impl InboxResolver {
    #[must_use]
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }

    pub async fn resolve(
        &self,
        post: &post::Model,
    ) -> Result<impl Stream<Item = Result<String, DbErr>> + Send + '_> {
        let account = post
            .find_related(account::Entity)
            .one(&self.db_conn)
            .await?
            .expect("[Bug] Post without associated account");

        let mentioned_inbox_stream = post
            .find_linked(mention::MentionedAccounts)
            .select_only()
            .column(account::Column::InboxUrl)
            .into_values::<String, InboxUrlQuery>()
            .stream(&self.db_conn)
            .await?;

        Ok(if post.visibility == Visibility::MentionOnly {
            Either::Left(mentioned_inbox_stream)
        } else {
            let follower_inbox_stream = account
                .find_linked(follow::Followers)
                .select_only()
                .column(account::Column::InboxUrl)
                .into_values::<_, InboxUrlQuery>()
                .stream(&self.db_conn)
                .await?;

            Either::Right(mentioned_inbox_stream.chain(follower_inbox_stream))
        })
    }
}
