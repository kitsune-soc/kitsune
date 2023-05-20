use crate::error::Result;
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods, QueryDsl};
use futures_util::{future::Either, Stream, StreamExt};
use kitsune_db::{
    function::coalesce_nullable,
    model::{
        account::Account,
        mention::Mention,
        post::{Post, Visibility},
    },
    PgPool,
};

pub struct InboxResolver {
    db_conn: PgPool,
}

impl InboxResolver {
    #[must_use]
    pub fn new(db_conn: PgPool) -> Self {
        Self { db_conn }
    }

    #[instrument(skip_all, fields(account_id = %account.id))]
    pub async fn resolve_followers(
        &self,
        account: &Account,
    ) -> Result<impl Stream<Item = Result<String>> + Send + '_> {
        use kitsune_db::schema::{accounts, accounts_follows};

        accounts_follows::table
            .filter(accounts_follows::account_id.eq(account.id))
            .inner_join(
                accounts::table.on(accounts::id.eq(accounts_follows::follower_id).and(
                    accounts::inbox_url
                        .is_not_null()
                        .or(accounts::shared_inbox_url.is_not_null()),
                )),
            )
            .select(coalesce_nullable(
                accounts::shared_inbox_url,
                accounts::inbox_url,
            ))
            .distinct()
            .load_stream(&self.db_conn)
            .await?
    }

    #[instrument(skip_all, fields(post_id = %post.id))]
    pub async fn resolve(
        &self,
        post: &Post,
    ) -> Result<impl Stream<Item = Result<String>> + Send + '_> {
        use kitsune_db::schema::accounts;

        let account = accounts::table
            .find(post.account_id)
            .first(&self.db_conn)
            .await?;

        let mentioned_inbox_stream = Mention::belonging_to(post)
            .inner_join(accounts::table)
            .filter(
                accounts::shared_inbox_url
                    .is_not_null()
                    .or(accounts::inbox_url.is_not_null()),
            )
            .select(coalesce_nullable(
                accounts::shared_inbox_url,
                accounts::inbox_url,
            ))
            .load_stream(&self.db_conn)
            .await?;

        let stream = if post.visibility == Visibility::MentionOnly {
            Either::Left(mentioned_inbox_stream)
        } else {
            Either::Right(mentioned_inbox_stream.chain(self.resolve_followers(&account).await?))
        };

        Ok(stream)
    }
}
