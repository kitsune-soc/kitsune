use crate::{config::FederationFilterConfiguration, error::Result};
use once_cell::unsync::OnceCell;
use regex::RegexSet;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct FederationFilterService {
    compiled_regexset: OnceCell<RegexSet>,
    config: FederationFilterConfiguration,
}

impl FederationFilterService {
    fn matches_rules(&self, domain: &str) -> Result<bool> {
        let regexset = self.compiled_regexset.get_or_try_init(|| {
            let regexes = match self.config {
                FederationFilterConfiguration::Allow { ref domains }
                | FederationFilterConfiguration::Deny { ref domains } => domains,
            }
            .iter()
            .map(|regex| format!("^{regex}$"));

            RegexSet::new(regexes)
        })?;

        Ok(regexset.is_match(domain))
    }
}
