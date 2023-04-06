use crate::{config::FederationFilterConfiguration, error::FederationFilterError};
use globset::{Glob, GlobSet, GlobSetBuilder};
use kitsune_type::ap::{object::Actor, Activity, Object};
use once_cell::unsync::OnceCell;
use std::ops::Not;
use typed_builder::TypedBuilder;
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

#[derive(TypedBuilder)]
pub struct FederationFilterService {
    compiled_regexset: OnceCell<GlobSet>,
    config: FederationFilterConfiguration,
}

impl FederationFilterService {
    fn matches_domain_rules(&self, domain: &str) -> Result<bool, FederationFilterError> {
        let globset = self.compiled_regexset.get_or_try_init(|| {
            let globs = match self.config {
                FederationFilterConfiguration::Allow { ref domains }
                | FederationFilterConfiguration::Deny { ref domains } => domains,
            };

            let mut globset = GlobSetBuilder::new();
            for glob in globs {
                globset.add(Glob::new(glob)?);
            }

            globset.build()
        })?;

        Ok(globset.is_match(domain))
    }

    pub fn is_entity_allowed<T>(&self, entity: &T) -> Result<bool, FederationFilterError>
    where
        T: Entity,
    {
        let id = Url::parse(entity.id())?;
        let host = id.host_str().ok_or(FederationFilterError::HostMissing)?;

        match self.config {
            FederationFilterConfiguration::Allow { .. } => self.matches_domain_rules(host),
            FederationFilterConfiguration::Deny { .. } => {
                self.matches_domain_rules(host).map(Not::not)
            }
        }
    }
}
