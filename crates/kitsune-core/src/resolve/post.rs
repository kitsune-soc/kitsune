use crate::{
    error::{Error, Result},
    service::account::{AccountService, GetUser},
};
use post_process::{BoxError, Element, Html, Render};
use speedy_uuid::Uuid;
use std::{borrow::Cow, sync::mpsc};
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct PostResolver {
    account: AccountService,
}

impl PostResolver {
    async fn transform<'a>(
        &self,
        element: Element<'a>,
        mentioned_accounts: mpsc::Sender<(Uuid, String)>,
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
                    let _ = mentioned_accounts.send((account.id, mention_text));

                    Element::Html(Html {
                        tag: Cow::Borrowed("a"),
                        attributes: vec![
                            (Cow::Borrowed("class"), Cow::Borrowed("mention")),
                            (Cow::Borrowed("href"), Cow::Owned(account.url)),
                        ],
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
    #[instrument(skip_all)]
    pub async fn resolve(&self, content: &str) -> Result<(Vec<(Uuid, String)>, String)> {
        let (mentioned_account_ids_acc, mentioned_account_ids) = mpsc::channel();

        let content = post_process::transform(content, |elem| {
            self.transform(elem, mentioned_account_ids_acc.clone())
        })
        .await
        .map_err(Error::PostProcessing)?;

        Ok((mentioned_account_ids.try_iter().collect(), content))
    }
}

#[cfg(test)]
mod test {
    use super::PostResolver;
    use crate::{
        activitypub::Fetcher,
        config::FederationFilterConfiguration,
        job::KitsuneContextRepo,
        service::{
            account::AccountService, attachment::AttachmentService,
            federation_filter::FederationFilterService, job::JobService, url::UrlService,
        },
        webfinger::Webfinger,
    };
    use athena::JobQueue;
    use core::convert::Infallible;
    use diesel::{QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use hyper::{Body, Request, Response};
    use kitsune_cache::NoopCache;
    use kitsune_db::{model::account::Account, schema::accounts};
    use kitsune_http_client::Client;
    use kitsune_search::NoopSearchService;
    use kitsune_storage::fs::Storage as FsStorage;
    use kitsune_test::{database_test, redis_test};
    use pretty_assertions::assert_eq;
    use scoped_futures::ScopedFutureExt;
    use std::sync::Arc;
    use tower::service_fn;

    #[tokio::test]
    #[serial_test::serial]
    async fn parse_mentions() {
        redis_test(|redis_pool| async move {
            database_test(|db_pool| async move {
                let post = "Hello @0x0@corteximplant.com! How are you doing?";

                let client = service_fn(|req: Request<_>| async move {
                    match req.uri().path_and_query().unwrap().as_str() {
                        "/.well-known/webfinger?resource=acct:0x0@corteximplant.com"=> {
                            let body = include_str!("../../../../test-fixtures/0x0_jrd.json");
                            Ok::<_, Infallible>(Response::new(Body::from(body)))
                        }
                        "/users/0x0" => {
                            let body = include_str!("../../../../test-fixtures/0x0_actor.json");
                            Ok::<_, Infallible>(Response::new(Body::from(body)))
                        }
                        path => panic!("HTTP client hit unexpected route: {path}"),
                    }
                });
                let client = Client::builder().service(client);

                let fetcher = Fetcher::builder()
                    .client(client)
                    .db_pool(db_pool.clone())
                    .embed_client(None)
                    .federation_filter(
                        FederationFilterService::new(&FederationFilterConfiguration::Deny {
                            domains: Vec::new(),
                        })
                        .unwrap(),
                    )
                    .search_service(NoopSearchService)
                    .webfinger(Webfinger::new(Arc::new(NoopCache.into())))
                    .post_cache(Arc::new(NoopCache.into()))
                    .user_cache(Arc::new(NoopCache.into()))
                    .build();

                let context_repo = KitsuneContextRepo::builder().db_pool(db_pool.clone()).build();
                let job_queue = JobQueue::builder()
                    .context_repository(context_repo)
                    .queue_name("parse_mentions_test")
                    .redis_pool(redis_pool)
                    .build();

                let job_service = JobService::builder().job_queue(job_queue).build();

                let url_service = UrlService::builder()
                    .domain("example.com")
                    .scheme("http")
                    .build();

                let attachment_service = AttachmentService::builder()
                    .db_pool(db_pool.clone())
                    .media_proxy_enabled(false)
                    .storage_backend(FsStorage::new("uploads".into()))
                    .url_service(url_service.clone())
                    .build();

                let account_service = AccountService::builder()
                    .attachment_service(attachment_service)
                    .db_pool(db_pool.clone())
                    .fetcher(fetcher)
                    .job_service(job_service)
                    .url_service(url_service.clone())
                    .webfinger(Webfinger::new(Arc::new(NoopCache.into())))
                    .build();

                let mention_resolver = PostResolver::builder()
                    .account(account_service)
                    .build();

                let (mentioned_account_ids, content) = mention_resolver
                    .resolve(post)
                    .await
                    .expect("Failed to resolve mentions");

                assert_eq!(content, "Hello <a class=\"mention\" href=\"https://corteximplant.com/users/0x0\">@0x0@corteximplant.com</a>! How are you doing?");
                assert_eq!(mentioned_account_ids.len(), 1);

                let (account_id, _mention_text) = &mentioned_account_ids[0];
                let mentioned_account = db_pool
                    .with_connection(|db_conn| {
                        accounts::table
                            .find(account_id)
                            .select(Account::as_select())
                            .get_result::<Account>(db_conn)
                            .scoped()
                    })
                    .await
                    .expect("Failed to fetch account");

                assert_eq!(mentioned_account.username, "0x0");
                assert_eq!(mentioned_account.domain, "corteximplant.com");
                assert_eq!(
                    mentioned_account.url,
                    "https://corteximplant.com/users/0x0"
                );
            }).await;
        }).await;
    }
}
