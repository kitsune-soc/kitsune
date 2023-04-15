use crate::{
    error::Result,
    job::{JobContext, Runnable},
};
use async_trait::async_trait;
use chrono::Utc;
use kitsune_db::entity::prelude::{Accounts, AccountsFollowers, Users};
use kitsune_type::ap::{ap_context, helper::StringOrObject, Activity, ActivityType, ObjectField};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct DeliverAccept {
    pub follow_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverAccept {
    #[instrument(skip_all, fields(follow_id = %self.follow_id))]
    async fn run(&self, ctx: JobContext<'_>) -> Result<()> {
        let Some(follow) = AccountsFollowers::find_by_id(self.follow_id)
            .one(&ctx.state.db_conn)
            .await?
        else {
            return Ok(());
        };
        let follower = Accounts::find_by_id(follow.follower_id)
            .one(&ctx.state.db_conn)
            .await?
            .expect("[Bug] Missing follower");
        let Some((followed_account, Some(followed_user))) = Accounts::find_by_id(follow.account_id)
            .find_also_related(Users)
            .one(&ctx.state.db_conn)
            .await?
        else {
            error!("missing followed user");
            return Ok(());
        };

        // Constructing this here is against our idea of the `IntoActivity` and `IntoObject` traits
        // But I'm not sure how I could encode these into the form of these two traits
        // So we make an exception for this
        //
        // If someone has a better idea, please open an issue
        let accept_activity = Activity {
            context: ap_context(),
            id: format!("{}#accept", follow.url),
            r#type: ActivityType::Accept,
            actor: StringOrObject::String(followed_account.url.clone()),
            object: ObjectField::Url(follow.url),
            published: Utc::now(),
        };

        ctx.deliverer
            .deliver(
                &follower.inbox_url,
                &followed_account,
                &followed_user,
                &accept_activity,
            )
            .await?;

        Ok(())
    }
}
