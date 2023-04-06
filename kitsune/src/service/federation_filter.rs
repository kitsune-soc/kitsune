use crate::{config::FederationFilterConfiguration, error::FederationFilterError};
use globset::{Glob, GlobSet, GlobSetBuilder};
use kitsune_type::ap::{object::Actor, Activity, Object};
use std::sync::Arc;
use url::Url;

pub trait Entity {
    fn id(&self) -> &str;
}

impl Entity for Activity {
    fn id(&self) -> &str {
        &self.rest.id
    }
}

impl Entity for Actor {
    fn id(&self) -> &str {
        &self.rest.id
    }
}

impl Entity for Object {
    fn id(&self) -> &str {
        &self.rest.id
    }
}

#[derive(Clone, Copy)]
enum FederationFilter {
    Allow,
    Deny,
}

#[derive(Clone)]
pub struct FederationFilterService {
    domains: Arc<GlobSet>,
    filter: FederationFilter,
}

impl FederationFilterService {
    pub fn new(config: &FederationFilterConfiguration) -> Result<Self, FederationFilterError> {
        let (filter, globs) = match config {
            FederationFilterConfiguration::Allow { ref domains } => {
                (FederationFilter::Allow, domains)
            }
            FederationFilterConfiguration::Deny { ref domains } => {
                (FederationFilter::Deny, domains)
            }
        };

        let mut globset = GlobSetBuilder::new();
        for glob in globs {
            globset.add(Glob::new(glob)?);
        }
        let domains = Arc::new(globset.build()?);

        Ok(Self { domains, filter })
    }

    pub fn is_url_allowed(&self, url: &Url) -> Result<bool, FederationFilterError> {
        let host = url.host_str().ok_or(FederationFilterError::HostMissing)?;

        let allowed = match self.filter {
            FederationFilter::Allow { .. } => self.domains.is_match(host),
            FederationFilter::Deny { .. } => !self.domains.is_match(host),
        };
        Ok(allowed)
    }

    pub fn is_entity_allowed<T>(&self, entity: &T) -> Result<bool, FederationFilterError>
    where
        T: Entity,
    {
        let id = Url::parse(entity.id())?;
        self.is_url_allowed(&id)
    }
}
