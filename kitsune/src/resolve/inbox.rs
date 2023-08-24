use crate::error::{Error, Result};
use diesel::{
    result::Error as DieselError, BelongingToDsl, BoolExpressionMethods, ExpressionMethods,
    JoinOnDsl, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use futures_util::{future::Either, Stream, StreamExt};
use kitsune_db::{
    function::coalesce_nullable,
    model::{
        account::Account,
        mention::Mention,
        post::{Post, Visibility},
    },
    schema::{accounts, accounts_follows},
    PgPool,
};

pub struct InboxResolver {
    db_pool: PgPool,
}

impl InboxResolver {
    #[must_use]
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    #[instrument(skip_all, fields(account_id = %account.id))]
    pub async fn resolve_followers(
        &self,
        account: &Account,
    ) -> Result<impl Stream<Item = Result<String, DieselError>> + Send + '_> {
        self.db_pool
            .with_connection(|mut db_conn| {
                accounts_follows::table
                    .filter(accounts_follows::account_id.eq(account.id))
                    .inner_join(
                        accounts::table.on(accounts::id.eq(accounts_follows::follower_id).and(
                            accounts::inbox_url
                                .is_not_null()
                                .or(accounts::shared_inbox_url.is_not_null()),
                        )),
                    )
                    .distinct()
                    .select(coalesce_nullable(
                        accounts::shared_inbox_url,
                        accounts::inbox_url,
                    ))
                    .load_stream(&mut db_conn)
            })
            .await
            .map_err(Error::from)
    }

    #[instrument(skip_all, fields(post_id = %post.id))]
    pub async fn resolve(
        &self,
        post: &Post,
    ) -> Result<impl Stream<Item = Result<String, DieselError>> + Send + '_> {
        let (account, mentioned_inbox_stream) = self
            .db_pool
            .with_connection(|mut db_conn| async move {
                let account = accounts::table
                    .find(post.account_id)
                    .select(Account::as_select())
                    .first(&mut db_conn)
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
                    .load_stream(&mut db_conn)
                    .await?;

                Ok::<_, Error>((account, mentioned_inbox_stream))
            })
            .await?;

        let stream = if post.visibility == Visibility::MentionOnly {
            Either::Left(mentioned_inbox_stream)
        } else {
            Either::Right(mentioned_inbox_stream.chain(self.resolve_followers(&account).await?))
        };

        Ok(stream)
    }
}
