use globset::{Glob, GlobSet, GlobSetBuilder};
use kitsune_config::instance::FederationFilterConfiguration;
use kitsune_derive::kitsune_service;
use kitsune_error::{kitsune_error, Result};
use kitsune_type::ap::{actor::Actor, Activity, Object};
use url::Url;

pub trait Entity {
    fn id(&self) -> &str;
}

impl Entity for Activity {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Entity for Actor {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Entity for Object {
    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Copy)]
enum FilterMode {
    Allow,
    Deny,
}

#[kitsune_service(omit_builder)]
pub struct FederationFilter {
    domains: GlobSet,
    filter: FilterMode,
}

impl FederationFilter {
    pub fn new(config: &FederationFilterConfiguration) -> Result<Self> {
        let (filter, globs) = match config {
            FederationFilterConfiguration::Allow { domains } => (FilterMode::Allow, domains),
            FederationFilterConfiguration::Deny { domains } => (FilterMode::Deny, domains),
        };

        let mut globset = GlobSetBuilder::new();
        for glob in globs {
            globset.add(Glob::new(glob)?);
        }

        Ok(__FederationFilter__Inner {
            domains: globset.build()?,
            filter,
        }
        .into())
    }

    pub fn is_url_allowed(&self, url: &Url) -> Result<bool> {
        let host = url
            .host_str()
            .ok_or_else(|| kitsune_error!("missing host component"))?;

        let allowed = match self.filter {
            FilterMode::Allow => self.domains.is_match(host),
            FilterMode::Deny => !self.domains.is_match(host),
        };
        Ok(allowed)
    }

    pub fn is_entity_allowed<T>(&self, entity: &T) -> Result<bool>
    where
        T: Entity,
    {
        let id = Url::parse(entity.id())?;
        self.is_url_allowed(&id)
    }
}
