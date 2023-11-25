use self::activitypub::prepare_activitypub;
use derive_more::From;
use futures_util::{future::BoxFuture, FutureExt, TryFutureExt};
use kitsune_activitypub::{Deliverer as ActivityPubDeliverer, Fetcher as ActivityPubFetcher};
use kitsune_core::{
    error::BoxError,
    traits::{deliverer, fetcher::AccountFetchOptions, Deliverer, Fetcher},
};
use kitsune_db::model::{account::Account, custom_emoji::CustomEmoji, post::Post};
use kitsune_webfinger::Webfinger;
use std::sync::Arc;
use typed_builder::TypedBuilder;

mod activitypub;

pub use self::activitypub::PrepareActivityPub;

pub struct Federator {
    pub deliverer: Vec<AnyDeliverer>,
    pub fetcher: Vec<AnyFetcher>,
}

#[derive(Clone, From)]
pub enum AnyDeliverer {
    ActivityPub(Arc<ActivityPubDeliverer>),
}

impl Deliverer for AnyDeliverer {
    type Error = BoxError;

    fn deliver(&self, action: deliverer::Action) -> BoxFuture<'_, Result<(), Self::Error>> {
        match self {
            Self::ActivityPub(deliverer) => deliverer.deliver(action).map_err(Into::into).boxed(),
        }
    }
}

#[derive(Clone, From)]
pub enum AnyFetcher {
    ActivityPub(ActivityPubFetcher<Webfinger>),
}

impl Fetcher for AnyFetcher {
    type Error = BoxError;

    async fn fetch_account(
        &self,
        opts: AccountFetchOptions<'_>,
    ) -> Result<Option<Account>, Self::Error> {
        match self {
            Self::ActivityPub(fetcher) => fetcher.fetch_account(opts).await.map_err(Into::into),
        }
    }

    async fn fetch_emoji(&self, url: &str) -> Result<Option<CustomEmoji>, Self::Error> {
        match self {
            Self::ActivityPub(fetcher) => fetcher.fetch_emoji(url).await.map_err(Into::into),
        }
    }

    async fn fetch_post(&self, url: &str) -> Result<Option<Post>, Self::Error> {
        match self {
            Self::ActivityPub(fetcher) => fetcher.fetch_post(url).await.map_err(Into::into),
        }
    }
}

#[derive(TypedBuilder)]
pub struct Prepare {
    activitypub: PrepareActivityPub,
}

pub fn prepare_federator(prepare: Prepare) -> Federator {
    let (fetcher, deliverer) = prepare_activitypub(prepare.activitypub);

    Federator {
        deliverer: vec![deliverer.into()],
        fetcher: vec![fetcher.into()],
    }
}
