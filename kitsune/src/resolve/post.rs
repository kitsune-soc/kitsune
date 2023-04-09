use crate::{
    error::{Error, Result},
    service::account::{AccountService, GetUser},
};
use parking_lot::Mutex;
use post_process::{BoxError, Element, Html, Render, Transformer};
use std::{borrow::Cow, collections::HashMap, mem};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Clone, TypedBuilder)]
pub struct PostResolver {
    account: AccountService,
}

impl PostResolver {
    async fn transform<'a>(
        &'a self,
        element: Element<'a>,
        mentioned_accounts: &Mutex<HashMap<Uuid, String>>,
    ) -> Result<Element<'a>, BoxError> {
        let element = match element {
            Element::Mention(mention) => {
                let get_user = GetUser::builder()
                    .username(&mention.username)
                    .domain(mention.domain.as_deref())
                    .build();

                if let Some(account) = self.account.get(get_user).await? {
                    let mut mention_text = String::new();
                    Element::Mention(mention.clone()).render(&mut mention_text);
                    mentioned_accounts.lock().insert(account.id, mention_text);

                    Element::Html(Html {
                        tag: Cow::Borrowed("a"),
                        attributes: vec![(Cow::Borrowed("href"), Cow::Owned(account.url))],
                        content: Box::new(Element::Mention(mention)),
                    })
                } else {
                    Element::Mention(mention)
                }
            }
            Element::Link(link) => Element::Html(Html {
                tag: Cow::Borrowed("a"),
                attributes: vec![(Cow::Borrowed("href"), link.content.clone())],
                content: Box::new(Element::Link(link)),
            }),
            elem => elem,
        };

        Ok(element)
    }

    /// Resolve the mentions inside a post
    ///
    /// # Returns
    ///
    /// - List of mentioned accounts, represented as `(Account ID, Mention text)`
    /// - Content with the mentions replaced by links
    ///
    /// # Panics
    ///
    /// This should never panic
    #[instrument(skip_all)]
    pub async fn resolve(&self, content: &str) -> Result<(Vec<(Uuid, String)>, String)> {
        let mentioned_account_ids = Mutex::new(HashMap::new());
        let transformer = Transformer::new(|elem| self.transform(elem, &mentioned_account_ids));

        let content = transformer
            .transform(content)
            .await
            .map_err(Error::PostProcessing)?;

        let mentioned_account_ids = mem::take(&mut *mentioned_account_ids.lock());
        Ok((mentioned_account_ids.into_iter().collect(), content))
    }
}

#[cfg(test)]
mod test {
    use super::PostResolver;
    use crate::{
        activitypub::Fetcher,
        cache::NoopCache,
        config::FederationFilterConfiguration,
        service::{
            account::AccountService, federation_filter::FederationFilterService, job::JobService,
            search::NoopSearchService, url::UrlService,
        },
        webfinger::Webfinger,
    };
    use kitsune_db::entity::prelude::Accounts;
    use pretty_assertions::assert_eq;
    use sea_orm::EntityTrait;
    use std::sync::Arc;

    #[tokio::test]
    async fn parse_mentions() {
        let db_conn = kitsune_db::connect("sqlite::memory:").await.unwrap();
        let post = "Hello @0x0@corteximplant.com! How are you doing?";

        let fetcher = Fetcher::builder()
            .db_conn(db_conn.clone())
            .federation_filter(
                FederationFilterService::new(&FederationFilterConfiguration::Deny {
                    domains: Vec::new(),
                })
                .unwrap(),
            )
            .search_service(NoopSearchService)
            .post_cache(Arc::new(NoopCache.into()))
            .user_cache(Arc::new(NoopCache.into()))
            .build();

        let job_service = JobService::builder().db_conn(db_conn.clone()).build();
        let url_service = UrlService::builder()
            .domain("example.com")
            .scheme("http")
            .build();
        let account_service = AccountService::builder()
            .db_conn(db_conn.clone())
            .fetcher(fetcher)
            .job_service(job_service)
            .url_service(url_service)
            .webfinger(Webfinger::new(Arc::new(NoopCache.into())))
            .build();
        let mention_resolver = PostResolver::builder().account(account_service).build();

        let (mentioned_account_ids, content) = mention_resolver
            .resolve(post)
            .await
            .expect("Failed to resolve mentions");

        assert_eq!(content, "Hello <a href=\"https://corteximplant.com/users/0x0\">@0x0@corteximplant.com</a>! How are you doing?");
        assert_eq!(mentioned_account_ids.len(), 1);

        let (account_id, _mention_text) = &mentioned_account_ids[0];
        let mentioned_account = Accounts::find_by_id(*account_id)
            .one(&db_conn)
            .await
            .ok()
            .flatten()
            .expect("Failed to fetch account");

        assert_eq!(mentioned_account.username, "0x0");
        assert_eq!(mentioned_account.domain, Some("corteximplant.com".into()));
        assert_eq!(mentioned_account.url, "https://corteximplant.com/users/0x0");
    }
}
