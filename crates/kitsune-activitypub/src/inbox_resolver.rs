use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper,
    result::Error as DieselError,
};
use diesel_async::RunQueryDsl;
use futures_util::{Stream, StreamExt, future::Either};
use kitsune_db::{
    PgPool,
    function::coalesce_nullable,
    model::{Account, Post},
    schema::{accounts, accounts_activitypub, accounts_follows, posts_mentions},
    types::Visibility,
    with_connection,
};
use kitsune_error::{Error, Result};

pub struct InboxResolver {
    db_pool: PgPool,
}

impl InboxResolver {
    #[must_use]
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    #[cfg_attr(not(coverage), instrument(skip_all, fields(account_id = %account.id)))]
    pub async fn resolve_followers(
        &self,
        account: &Account,
    ) -> Result<impl Stream<Item = Result<String, DieselError>> + Send + use<'_>> {
        with_connection!(self.db_pool, |db_conn| {
            accounts_follows::table
                .filter(accounts_follows::account_id.eq(account.id))
                .inner_join(accounts::table.on(accounts::id.eq(accounts_follows::follower_id)))
                .inner_join(
                    accounts_activitypub::table
                        .on(accounts::id.eq(accounts_activitypub::account_id)),
                )
                .filter(
                    accounts_activitypub::inbox_url
                        .is_not_null()
                        .or(accounts_activitypub::shared_inbox_url.is_not_null()),
                )
                .distinct()
                .select(coalesce_nullable(
                    accounts_activitypub::shared_inbox_url,
                    accounts_activitypub::inbox_url,
                ))
                .load_stream(db_conn)
                .await
        })
        .map_err(Error::from)
    }

    #[cfg_attr(not(coverage), instrument(skip_all, fields(post_id = %post.id)))]
    pub async fn resolve(
        &self,
        post: &Post,
    ) -> Result<impl Stream<Item = Result<String, DieselError>> + Send + use<'_>> {
        let (account, mentioned_inbox_stream) = with_connection!(self.db_pool, |db_conn| {
            let account = accounts::table
                .find(post.account_id)
                .select(Account::as_select())
                .first(db_conn)
                .await?;

            let mentioned_inbox_stream = posts_mentions::table
                .filter(posts_mentions::post_id.eq(post.id))
                .inner_join(accounts::table.on(posts_mentions::account_id.eq(accounts::id)))
                .inner_join(
                    accounts_activitypub::table
                        .on(accounts::id.eq(accounts_activitypub::account_id)),
                )
                .filter(
                    accounts_activitypub::shared_inbox_url
                        .is_not_null()
                        .or(accounts_activitypub::inbox_url.is_not_null()),
                )
                .select(coalesce_nullable(
                    accounts_activitypub::shared_inbox_url,
                    accounts_activitypub::inbox_url,
                ))
                .load_stream(db_conn)
                .await?;

            Ok::<_, Error>((account, mentioned_inbox_stream))
        })?;

        let stream = if post.visibility == Visibility::MentionOnly {
            Either::Left(mentioned_inbox_stream)
        } else {
            Either::Right(mentioned_inbox_stream.chain(self.resolve_followers(&account).await?))
        };

        Ok(stream)
    }
}
