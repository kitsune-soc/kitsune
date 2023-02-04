use crate::{activitypub::Deliverer, error::Result, mapping::IntoActivity, state::Zustand};
use kitsune_db::{
    column::InboxUrlQuery,
    entity::{accounts, favourites, users},
    link::FavouritedPostAuthor,
};
use sea_orm::{prelude::*, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct UnfavouriteDeliveryContext {
    pub favourite_id: Uuid,
}

pub async fn run(
    state: &Zustand,
    deliverer: &Deliverer,
    ctx: UnfavouriteDeliveryContext,
) -> Result<()> {
    let Some(favourite) = favourites::Entity::find_by_id(ctx.favourite_id)
        .one(&state.db_conn)
        .await?
    else {
        return Ok(());
    };

    let Some((account, Some(user))) = favourite
        .find_related(accounts::Entity)
        .find_also_related(users::Entity)
        .one(&state.db_conn)
        .await?
    else {
        return Ok(());
    };

    favourites::Entity::delete_by_id(favourite.id)
        .exec(&state.db_conn)
        .await?;

    let inbox_url = favourite
        .find_linked(FavouritedPostAuthor)
        .select_only()
        .column(accounts::Column::InboxUrl)
        .into_values::<String, InboxUrlQuery>()
        .one(&state.db_conn)
        .await?
        .expect("[Bug] Post without associated account");
    let activity = favourite.into_negate_activity(state).await?;

    // TODO: Maybe deliver to followers as well?
    deliverer
        .deliver(&inbox_url, &account, &user, &activity)
        .await?;

    Ok(())
}
