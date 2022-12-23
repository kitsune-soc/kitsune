use sea_orm::{prelude::*, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    activitypub::Deliverer,
    db::model::{account, favourite, user},
    error::Result,
    mapping::IntoActivity,
    state::Zustand,
};

#[derive(Deserialize, Serialize)]
pub struct FavouriteDeliveryContext {
    pub favourite_id: Uuid,
}

#[derive(Copy, Clone, Debug, DeriveColumn, EnumIter)]
enum InboxUrlQuery {
    InboxUrl,
}

pub async fn run(
    state: &Zustand,
    deliverer: &Deliverer,
    ctx: FavouriteDeliveryContext,
) -> Result<()> {
    let Some(favourite) = favourite::Entity::find_by_id(ctx.favourite_id)
        .one(&state.db_conn)
        .await?
    else {
        return Ok(());
    };

    let Some((account, Some(user))) = favourite
        .find_related(account::Entity)
        .find_also_related(user::Entity)
        .one(&state.db_conn)
        .await?
    else {
        return Ok(());
    };

    let inbox_url = favourite
        .find_linked(favourite::FavouritedPostAuthor)
        .select_only()
        .column(account::Column::InboxUrl)
        .into_values::<String, InboxUrlQuery>()
        .one(&state.db_conn)
        .await?
        .expect("[Bug] Post without associated account");
    let activity = favourite.into_activity(state).await?;

    // TODO: Maybe deliver to followers as well?
    deliverer
        .deliver(&inbox_url, &account, &user, &activity)
        .await?;

    Ok(())
}
