use crate::{
    activitypub::Deliverer,
    db::model::{account, post, user},
    error::{Error, Result},
    state::Zustand,
};
use futures_util::{stream, StreamExt};
use phenomenon_model::ap::{Activity, PUBLIC_IDENTIFIER};
use sea_orm::{EntityTrait, ModelTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_CONCURRENT_REQUESTS: usize = 10;

#[derive(Deserialize, Serialize)]
pub struct DeliveryContext {
    post_id: Uuid,
}

#[instrument(skip_all, fields(post_id = %ctx.post_id))]
pub async fn run(state: &Zustand, deliverer: &Deliverer, ctx: DeliveryContext) -> Result<()> {
    let Some((post, Some(account))) = post::Entity::find_by_id(ctx.post_id)
        .find_also_related(account::Entity)
        .one(&state.db_conn)
        .await?
    else {
        return Ok(());
    };
    let user = account
        .find_related(user::Entity)
        .one(&state.db_conn)
        .await?
        .expect("[Bug] Trying to deliver activity for account with no associated user");

    // TODO: Resolve follower collection
    // TODO: Actually fill this activity with meaningful data
    // TODO: Make more efficient for larger number of followers
    let activity: Activity = todo!();
    let delivery_futures_iter = activity
        .rest
        .to
        .iter()
        .cloned()
        .chain(activity.rest.cc.iter().cloned())
        .filter(|url| *url != PUBLIC_IDENTIFIER)
        .map(|ap_id| {
            let account = &account;
            let user = &user;
            let activity = &activity;

            async move {
                let account = state.fetcher.fetch_actor(&ap_id).await?;
                deliverer
                    .deliver(&account.inbox_url, &account, user, activity)
                    .await?;

                Ok::<_, Error>(())
            }
        });

    let mut delivery_results =
        stream::iter(delivery_futures_iter).buffer_unordered(MAX_CONCURRENT_REQUESTS);

    while let Some(delivery_result) = delivery_results.next().await {
        if let Err(err) = delivery_result {
            error!(error = %err, "Failed to deliver activity");
        }
    }

    Ok(())
}
