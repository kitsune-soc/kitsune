use crate::job::MAX_CONCURRENT_REQUESTS;
use crate::{
    activitypub::Deliverer, error::Result, mapping::IntoActivity, resolve::InboxResolver,
    state::Zustand,
};
use futures_util::TryStreamExt;
use kitsune_db::entity::prelude::{Accounts, Posts, Users};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct DeleteDeliveryContext {
    pub post_id: Uuid,
}

pub async fn run(state: &Zustand, deliverer: &Deliverer, ctx: DeleteDeliveryContext) -> Result<()> {
    let Some(post) = Posts::find_by_id(ctx.post_id)
        .one(&state.db_conn)
        .await?
    else {
        return Ok(());
    };

    let Some((account, Some(user))) = Accounts::find_by_id(post.account_id)
        .find_also_related(Users)
        .one(&state.db_conn)
        .await?
    else {
        return Ok(());
    };

    let inbox_resolver = InboxResolver::new(state.db_conn.clone());
    let inbox_stream = inbox_resolver
        .resolve(&post)
        .await?
        .try_chunks(MAX_CONCURRENT_REQUESTS)
        .map_err(|err| err.1);

    Posts::delete_by_id(post.id).exec(&state.db_conn).await?;

    let delete_activity = post.into_negate_activity(state).await?;

    // TODO: Should we deliver to the inboxes that are contained inside a `TryChunksError`?
    deliverer
        .deliver_many(&account, &user, &delete_activity, inbox_stream)
        .await?;

    Ok(())
}
