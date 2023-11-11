use crate::{
    error::Result,
    job::{JobRunnerContext, Runnable},
};
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl,
    SelectableHelper,
};
use diesel_async::RunQueryDsl;
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    model::{account::Account, follower::Follow, user::User},
    schema::{accounts, accounts_follows, users},
};
use kitsune_type::ap::{ap_context, helper::StringOrObject, Activity, ActivityType, ObjectField};
use kitsune_util::try_join;
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverReject {
    pub follow_id: Uuid,
}

impl Runnable for DeliverReject {
    type Context = JobRunnerContext;
    type Error = eyre::Report;

    #[instrument(skip_all, fields(follow_id = %self.follow_id))]
    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let follow = ctx
            .state
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    accounts_follows::table
                        .find(self.follow_id)
                        .get_result::<Follow>(db_conn)
                        .await
                        .optional()
                }
                .scoped()
            })
            .await?;

        let Some(follow) = follow else {
            return Ok(());
        };

        let (follower_inbox_url, (followed_account, followed_user), _delete_result) = ctx
            .state
            .db_pool
            .with_connection(|db_conn| {
                async {
                    let follower_inbox_url_fut = accounts::table
                        .find(follow.follower_id)
                        .select(accounts::inbox_url.assume_not_null())
                        .get_result::<String>(db_conn);

                    let followed_info_fut = accounts::table
                        .find(follow.account_id)
                        .inner_join(users::table.on(accounts::id.eq(users::account_id)))
                        .select(<(Account, User)>::as_select())
                        .get_result::<(Account, User)>(db_conn);

                    let delete_fut = diesel::delete(&follow).execute(db_conn);

                    try_join!(follower_inbox_url_fut, followed_info_fut, delete_fut)
                }
                .scoped()
            })
            .await?;

        let followed_account_url = ctx.state.service.url.user_url(followed_account.id);

        // Constructing this here is against our idea of the `IntoActivity` and `IntoObject` traits
        // But I'm not sure how I could encode these into the form of these two traits
        // So we make an exception for this
        //
        // If someone has a better idea, please open an issue
        let reject_activity = Activity {
            context: ap_context(),
            id: format!("{}#reject", follow.url),
            r#type: ActivityType::Reject,
            actor: StringOrObject::String(followed_account_url),
            object: ObjectField::Url(follow.url),
            published: Timestamp::now_utc(),
        };

        ctx.deliverer
            .deliver(
                &follower_inbox_url,
                &followed_account,
                &followed_user,
                &reject_activity,
            )
            .await?;

        Ok(())
    }
}
