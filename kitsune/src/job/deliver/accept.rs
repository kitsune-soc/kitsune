use crate::{
    error::Result,
    job::{JobContext, Runnable},
    try_join,
};
use async_trait::async_trait;
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
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverAccept {
    pub follow_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverAccept {
    #[instrument(skip_all, fields(follow_id = %self.follow_id))]
    async fn run(&self, ctx: JobContext<'_>) -> Result<()> {
        let mut db_conn = ctx.state.db_conn.get().await?;
        let Some(follow) = accounts_follows::table.find(self.follow_id)
            .get_result::<Follow>(&mut db_conn)
            .await
            .optional()?
        else {
            return Ok(());
        };

        let follower_inbox_url_fut = accounts::table
            .find(follow.follower_id)
            .select(accounts::inbox_url.assume_not_null())
            .get_result::<String>(&mut db_conn);

        let followed_info_fut = accounts::table
            .find(follow.account_id)
            .inner_join(users::table.on(accounts::id.eq(users::account_id)))
            .select((Account::as_select(), User::as_select()))
            .get_result::<(Account, User)>(&mut db_conn);

        let (follower_inbox_url, (followed_account, followed_user)) =
            try_join!(follower_inbox_url_fut, followed_info_fut)?;

        let followed_account_url = ctx.state.service.url.user_url(followed_account.id);

        // Constructing this here is against our idea of the `IntoActivity` and `IntoObject` traits
        // But I'm not sure how I could encode these into the form of these two traits
        // So we make an exception for this
        //
        // If someone has a better idea, please open an issue
        let accept_activity = Activity {
            context: ap_context(),
            id: format!("{}#accept", follow.url),
            r#type: ActivityType::Accept,
            actor: StringOrObject::String(followed_account_url),
            object: ObjectField::Url(follow.url),
            published: Timestamp::now_utc(),
        };

        ctx.deliverer
            .deliver(
                &follower_inbox_url,
                &followed_account,
                &followed_user,
                &accept_activity,
            )
            .await?;

        Ok(())
    }
}
