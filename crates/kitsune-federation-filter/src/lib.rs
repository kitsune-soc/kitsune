#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use crate::error::{Error, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use kitsune_config::instance::FederationFilterConfiguration;
use kitsune_type::ap::{actor::Actor, Activity, Object};
use std::sync::Arc;
use url::Url;

pub mod error;

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

#[derive(Clone)]
pub struct FederationFilter {
    domains: Arc<GlobSet>,
    filter: FilterMode,
}

impl FederationFilter {
    pub fn new(config: &FederationFilterConfiguration) -> Result<Self> {
        let (filter, globs) = match config {
            FederationFilterConfiguration::Allow { ref domains } => (FilterMode::Allow, domains),
            FederationFilterConfiguration::Deny { ref domains } => (FilterMode::Deny, domains),
        };

        let mut globset = GlobSetBuilder::new();
        for glob in globs {
            globset.add(Glob::new(glob)?);
        }
        let domains = Arc::new(globset.build()?);

        Ok(Self { domains, filter })
    }

    pub fn is_url_allowed(&self, url: &Url) -> Result<bool> {
        let host = url.host_str().ok_or(Error::HostMissing)?;

        let allowed = match self.filter {
            FilterMode::Allow { .. } => self.domains.is_match(host),
            FilterMode::Deny { .. } => !self.domains.is_match(host),
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
