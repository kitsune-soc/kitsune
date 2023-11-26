use crate::error::BoxError;
use futures_util::{future::BoxFuture, FutureExt};
use kitsune_db::model::{account::Account, favourite::Favourite, follower::Follow, post::Post};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub enum Action {
    AcceptFollow(Follow),
    Create(Post),
    Delete(Post),
    Favourite(Favourite),
    Follow(Follow),
    RejectFollow(Follow),
    Repost(Post),
    Unfavourite(Favourite),
    Unfollow(Follow),
    Unrepost(Post),
    UpdateAccount(Account),
    UpdatePost(Post),
}

pub trait Deliverer: Send + Sync + 'static {
    type Error: Into<BoxError>;

    fn deliver(&self, action: Action) -> BoxFuture<'_, Result<(), Self::Error>>;
}

impl<T> Deliverer for Vec<T>
where
    T: Deliverer,
{
    type Error = BoxError;

    fn deliver(&self, action: Action) -> BoxFuture<'_, Result<(), Self::Error>> {
        async move {
            for deliverer in self {
                deliverer
                    .deliver(action.clone())
                    .await
                    .map_err(Into::into)?;
            }

            Ok(())
        }
        .boxed()
    }
}
