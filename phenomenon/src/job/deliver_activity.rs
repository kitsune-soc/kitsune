use crate::{
    db::entity::{post, user},
    deliverer::Deliverer,
    error::{Error, Result},
    mapping::IntoActivityPub,
    state::Zustand,
};
use futures_util::{stream, StreamExt};
use phenomenon_model::ap::{Activity, PUBLIC_IDENTIFIER};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_CONCURRENT_REQUESTS: usize = 10;

#[derive(Deserialize, Serialize)]
pub struct DeliveryContext {
    post_id: Uuid,
}

#[instrument(skip_all, fields(post_id = %ctx.post_id))]
pub async fn run(state: &Zustand, deliverer: &Deliverer, ctx: DeliveryContext) -> Result<()> {
    let Some((post, Some(author))) = post::Entity::find_by_id(ctx.post_id)
        .find_also_related(user::Entity)
        .one(&state.db_conn)
        .await?
    else {
        return Ok(());
    };

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
            let author = &author;
            let activity = &activity;
            async move {
                let user = state.fetcher.fetch_actor(&ap_id).await?;
                deliverer.deliver(&user.inbox_url, author, activity).await?;

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
