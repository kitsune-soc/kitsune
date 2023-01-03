use crate::job::MAX_CONCURRENT_REQUESTS;
use crate::{
    activitypub::Deliverer,
    db::model::{account, post, user},
    error::Result,
    mapping::IntoActivity,
    resolve::InboxResolver,
    state::Zustand,
};
use futures_util::TryStreamExt;
use sea_orm::{EntityTrait, ModelTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct CreateDeliveryContext {
    pub post_id: Uuid,
}

#[instrument(skip_all, fields(post_id = %ctx.post_id))]
pub async fn run(state: &Zustand, deliverer: &Deliverer, ctx: CreateDeliveryContext) -> Result<()> {
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

    let inbox_resolver = InboxResolver::new(state.db_conn.clone());
    let inbox_stream = inbox_resolver
        .resolve(&post)
        .await?
        .try_chunks(MAX_CONCURRENT_REQUESTS)
        .map_err(|err| err.1);

    let activity = post.into_activity(state).await?;

    // TODO: Should we deliver to the inboxes that are contained inside a `TryChunksError`?
    deliverer
        .deliver_many(&account, &user, &activity, inbox_stream)
        .await?;

    Ok(())
}
