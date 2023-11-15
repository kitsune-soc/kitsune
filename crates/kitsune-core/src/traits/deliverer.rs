use crate::error::BoxError;
use kitsune_db::model::{account::Account, favourite::Favourite, follower::Follow, post::Post};
use serde::{Deserialize, Serialize};
use std::future::Future;

#[derive(Clone, Deserialize, Serialize)]
pub enum Action {
    AcceptFollow(Follow),
    Create(Post),
    Delete(Post),
    Favourite(Post),
    Follow(Follow),
    RejectFollow(Follow),
    Repost(Post),
    Unfavourite(Favourite),
    Unrepost(Post),
    UpdateAccount(Account),
    UpdatePost(Post),
}

pub trait Deliverer: Send + Sync + 'static {
    type Error: Into<BoxError>;

    fn deliver(&self, action: Action) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

impl<T> Deliverer for [T]
where
    T: Deliverer,
{
    type Error = BoxError;

    async fn deliver(&self, action: Action) -> Result<(), Self::Error> {
        for deliverer in self {
            deliverer
                .deliver(action.clone())
                .await
                .map_err(Into::into)?;
        }

        Ok(())
    }
}
