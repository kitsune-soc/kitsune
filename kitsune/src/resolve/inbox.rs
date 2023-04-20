use crate::error::{Error, Result};
use futures_util::{future::Either, Stream, StreamExt};
use kitsune_db::{
    column::InboxUrlQuery,
    custom::Visibility,
    entity::{accounts, posts, prelude::Accounts},
    link::{Followers, MentionedAccounts},
};
use sea_orm::{
    sea_query::{Expr, Func, SimpleExpr},
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter, QuerySelect,
};

pub struct InboxResolver {
    db_conn: DatabaseConnection,
}

impl InboxResolver {
    #[must_use]
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }

    #[instrument(skip_all, fields(account_id = %account.id))]
    pub async fn resolve_followers(
        &self,
        account: &accounts::Model,
    ) -> Result<impl Stream<Item = Result<String, DbErr>> + Send + '_> {
        account
            .find_linked(Followers)
            .filter(
                accounts::Column::SharedInboxUrl
                    .is_not_null()
                    .or(accounts::Column::InboxUrl.is_not_null()),
            )
            .select_only()
            .distinct()
            .column_as(
                SimpleExpr::from(Func::coalesce([
                    Expr::col(accounts::Column::SharedInboxUrl).into(),
                    Expr::col(accounts::Column::InboxUrl).into(),
                ])),
                accounts::Column::InboxUrl,
            )
            .into_values::<_, InboxUrlQuery>()
            .stream(&self.db_conn)
            .await
            .map_err(Error::from)
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
            .filter(
                accounts::Column::SharedInboxUrl
                    .is_not_null()
                    .or(accounts::Column::InboxUrl.is_not_null()),
            )
            .select_only()
            .distinct()
            .column_as(
                SimpleExpr::from(Func::coalesce([
                    Expr::col(accounts::Column::SharedInboxUrl).into(),
                    Expr::col(accounts::Column::InboxUrl).into(),
                ])),
                accounts::Column::InboxUrl,
            )
            .into_values::<String, InboxUrlQuery>()
            .stream(&self.db_conn)
            .await?;

        let stream = if post.visibility == Visibility::MentionOnly {
            Either::Left(mentioned_inbox_stream)
        } else {
            Either::Right(mentioned_inbox_stream.chain(self.resolve_followers(&account).await?))
        };

        Ok(stream)
    }
}
