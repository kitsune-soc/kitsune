use kitsune_core::traits::{Deliverer, Fetcher};
use std::sync::Arc;
use typed_builder::TypedBuilder;

pub mod activitypub;

pub struct Federator {
    pub deliverer: Arc<dyn Deliverer>,
    pub fetcher: Arc<dyn Fetcher>,
}

#[derive(TypedBuilder)]
pub struct PrepareDeliverer {
    activitypub: self::activitypub::PrepareDeliverer,
}

#[derive(TypedBuilder)]
pub struct PrepareFetcher {
    activitypub: self::activitypub::PrepareFetcher,
}

#[derive(TypedBuilder)]
pub struct Prepare {
    deliverer: PrepareDeliverer,
    fetcher: PrepareFetcher,
}

#[inline]
#[must_use]
pub fn prepare_deliverer(prepare: PrepareDeliverer) -> Arc<dyn Deliverer> {
    let deliverer = self::activitypub::prepare_deliverer(prepare.activitypub);
    Arc::new(vec![deliverer])
}

#[inline]
#[must_use]
pub fn prepare_fetcher(prepare: PrepareFetcher) -> Arc<dyn Fetcher> {
    let fetcher = self::activitypub::prepare_fetcher(prepare.activitypub);
    Arc::new(vec![fetcher])
}

#[inline]
#[must_use]
pub fn prepare_federator(prepare: Prepare) -> Federator {
    Federator {
        deliverer: prepare_deliverer(prepare.deliverer),
        fetcher: prepare_fetcher(prepare.fetcher),
    }
}
