use self::any::{AnyDeliverer, AnyFetcher};
use typed_builder::TypedBuilder;

pub mod activitypub;
pub mod any;

pub struct Federator {
    pub deliverer: Vec<AnyDeliverer>,
    pub fetcher: Vec<AnyFetcher>,
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
pub fn prepare_deliverer(prepare: PrepareDeliverer) -> Vec<AnyDeliverer> {
    let deliverer = self::activitypub::prepare_deliverer(prepare.activitypub);
    vec![deliverer.into()]
}

#[inline]
pub fn prepare_fetcher(prepare: PrepareFetcher) -> Vec<AnyFetcher> {
    let fetcher = self::activitypub::prepare_fetcher(prepare.activitypub);
    vec![fetcher.into()]
}

#[inline]
pub fn prepare_federator(prepare: Prepare) -> Federator {
    Federator {
        deliverer: prepare_deliverer(prepare.deliverer),
        fetcher: prepare_fetcher(prepare.fetcher),
    }
}
