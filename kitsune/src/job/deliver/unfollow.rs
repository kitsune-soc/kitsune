use crate::{
    error::Result,
    job::{JobContext, Runnable},
    mapping::IntoActivity,
};
use async_trait::async_trait;
use kitsune_db::entity::prelude::{Accounts, AccountsFollowers, Users};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct DeliverUnfollow {
    follow_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverUnfollow {
    async fn run(&self, ctx: JobContext<'_>) -> Result<()> {
        let Some(follow) = AccountsFollowers::find_by_id(self.follow_id)
            .one(&ctx.state.db_conn)
            .await?
        else {
            return Ok(());
        };

        let (follower, Some(follower_user)) = Accounts::find_by_id(follow.follower_id)
            .find_also_related(Users)
            .one(&ctx.state.db_conn)
            .await?
            .expect("[Bug] Follow without follower account")
        else {
            error!("Enqueued follow job for remote user");
            return Ok(());
        };

        let followed_account = Accounts::find_by_id(follow.account_id)
            .one(&ctx.state.db_conn)
            .await?
            .expect("[Bug] Follow without followed account");

        AccountsFollowers::delete_by_id(follow.id)
            .exec(&ctx.state.db_conn)
            .await?;

        let follow_activity = follow.into_negate_activity(ctx.state).await?;

        ctx.deliverer
            .deliver(
                &followed_account.inbox_url,
                &follower,
                &follower_user,
                &follow_activity,
            )
            .await?;

        Ok(())
    }
}
