use crate::{
    activitypub::Deliverer,
    db::model::{
        account, follow, mention,
        post::{self, Visibility},
        user,
    },
    error::Result,
    mapping::IntoActivityPub,
    state::Zustand,
};
use chrono::Utc;
use futures_util::{future::Either, stream::FuturesUnordered, StreamExt, TryStreamExt};
use phenomenon_type::ap::{helper::StringOrObject, Activity, ActivityType, BaseObject};
use sea_orm::{prelude::*, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_CONCURRENT_REQUESTS: usize = 10;

#[derive(Copy, Clone, Debug, DeriveColumn, EnumIter)]
enum InboxQuery {
    InboxUrl,
}

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

    let object = post.clone().into_activitypub(state).await?;
    let activity = Activity {
        r#type: ActivityType::Create,
        rest: BaseObject {
            id: format!("{}#create", object.id()),
            attributed_to: Some(StringOrObject::String(account.url.clone())),
            published: Utc::now(),
            to: object.to().to_vec(),
            cc: object.cc().to_vec(),
            ..Default::default()
        },
        object: StringOrObject::Object(object),
    };

    let mentioned_inbox_stream = post
        .find_linked(mention::MentionedAccounts)
        .select_only()
        .column(account::Column::InboxUrl)
        .into_values::<String, InboxQuery>()
        .stream(&state.db_conn)
        .await?;

    let mut inbox_stream = if post.visibility == Visibility::MentionOnly {
        Either::Left(mentioned_inbox_stream)
    } else {
        let follower_inbox_stream = account
            .find_linked(follow::Followers)
            .select_only()
            .column(account::Column::InboxUrl)
            .into_values::<_, InboxQuery>()
            .stream(&state.db_conn)
            .await?;

        Either::Right(mentioned_inbox_stream.chain(follower_inbox_stream))
    }
    .try_chunks(MAX_CONCURRENT_REQUESTS);

    // TODO: Should we deliver to the inboxes that are contained inside a `TryChunksError`?
    while let Some(inbox_chunk) = inbox_stream.next().await.transpose().map_err(|err| err.1)? {
        let mut concurrent_resolver: FuturesUnordered<_> = inbox_chunk
            .iter()
            .map(|inbox| deliverer.deliver(inbox, &account, &user, &activity))
            .collect();

        while let Some(delivery_result) = concurrent_resolver.next().await {
            if let Err(err) = delivery_result {
                error!(error = %err, "Failed to deliver activity to inbox");
            }
        }
    }

    Ok(())
}
