use self::deliver::{
    accept::DeliverAccept, create::DeliverCreate, delete::DeliverDelete,
    favourite::DeliverFavourite, follow::DeliverFollow, unfavourite::DeliverUnfavourite,
    unfollow::DeliverUnfollow, update::DeliverUpdate,
};
use async_trait::async_trait;
use athena::Runnable;
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod deliver;

const EXECUTION_TIMEOUT_DURATION: Duration = Duration::from_secs(30);
const MAX_CONCURRENT_REQUESTS: usize = 10;

macro_rules! impl_from {
    (
        $(#[$top_annotation:meta])*
        $vb:vis enum $name:ident {
        $(
            $(#[$branch_annotation:meta])*
            $branch_name:ident ($from_type:ty)
        ),+
        $(,)*
    }) => {
        $(#[$top_annotation])*
        $vb enum $name {
            $(
                $(#[$branch_annotation])*
                $branch_name($from_type),
            )*
        }

        $(
            impl From<$from_type> for $name {
                fn from(val: $from_type) -> Self {
                    Self::$branch_name(val)
                }
            }
        )*
    };
}

impl_from! {
    #[derive(Debug, Deserialize, Serialize)]
    pub enum Job {
        DeliverAccept(DeliverAccept),
        DeliverCreate(DeliverCreate),
        DeliverDelete(DeliverDelete),
        DeliverFavourite(DeliverFavourite),
        DeliverFollow(DeliverFollow),
        DeliverUnfavourite(DeliverUnfavourite),
        DeliverUnfollow(DeliverUnfollow),
        DeliverUpdate(DeliverUpdate),
    }
}

#[async_trait]
impl Runnable for Job {
    type Error = anyhow::Error;

    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        match self {
            Self::DeliverAccept(job) => job.run(ctx).await,
            Self::DeliverCreate(job) => job.run(ctx).await,
            Self::DeliverDelete(job) => job.run(ctx).await,
            Self::DeliverFavourite(job) => job.run(ctx).await,
            Self::DeliverFollow(job) => job.run(ctx).await,
            Self::DeliverUnfavourite(job) => job.run(ctx).await,
            Self::DeliverUnfollow(job) => job.run(ctx).await,
            Self::DeliverUpdate(job) => job.run(ctx).await,
        }
    }
}
