use async_trait::async_trait;
use eyre::Result;
use kitsune_db::model::{account::Account, favourite::Favourite, follower::Follow, post::Post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[async_trait]
pub trait Deliverer: Send + Sync + 'static {
    async fn deliver(&self, action: Action) -> Result<()>;
}

#[async_trait]
impl Deliverer for Arc<dyn Deliverer> {
    async fn deliver(&self, action: Action) -> Result<()> {
        (**self).deliver(action).await
    }
}

#[async_trait]
impl<T> Deliverer for Vec<T>
where
    T: Deliverer,
{
    async fn deliver(&self, action: Action) -> Result<()> {
        for deliverer in self {
            deliverer.deliver(action.clone()).await?;
        }

        Ok(())
    }
}
