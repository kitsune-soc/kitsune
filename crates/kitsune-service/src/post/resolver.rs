use crate::{
    account::{AccountService, GetUser},
    custom_emoji::{CustomEmojiService, GetEmoji},
    error::{Error, Result},
};
use post_process::{BoxError, Element, Html, Render};
use speedy_uuid::Uuid;
use std::{borrow::Cow, sync::mpsc};
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct PostResolver {
    account: AccountService,
    custom_emoji: CustomEmojiService,
}

pub struct ResolvedPost {
    pub mentioned_accounts: Vec<(Uuid, String)>,
    pub custom_emojis: Vec<(Uuid, String)>,
    pub content: String,
}

impl PostResolver {
    async fn transform<'a>(
        &self,
        element: Element<'a>,
        mentioned_accounts: mpsc::Sender<(Uuid, String)>,
        custom_emojis: mpsc::Sender<(Uuid, String)>,
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
            Element::Emote(emote) => {
                let get_emoji = GetEmoji::builder()
                    .shortcode(&emote.shortcode)
                    .domain(emote.domain.as_deref())
                    .build();

                if let Some(emoji) = self.custom_emoji.get(get_emoji).await? {
                    let mut emoji_text = String::new();
                    Element::Emote(emote.clone()).render(&mut emoji_text);
                    let _ = custom_emojis.send((emoji.id, emoji_text));
                }
                Element::Emote(emote)
            }
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
    pub async fn resolve(&self, content: &str) -> Result<ResolvedPost> {
        let (mentioned_account_ids_acc, mentioned_account_ids) = mpsc::channel();
        let (custom_emoji_ids_sen, custom_emoji_ids_rec) = mpsc::channel();

        let content = post_process::transform(content, |elem| {
            self.transform(
                elem,
                mentioned_account_ids_acc.clone(),
                custom_emoji_ids_sen.clone(),
            )
        })
        .await
        .map_err(Error::PostProcessing)?;

        Ok(ResolvedPost {
            mentioned_accounts: mentioned_account_ids.try_iter().collect(),
            custom_emojis: custom_emoji_ids_rec.try_iter().collect(),
            content,
        })
    }
}

#[cfg(test)]
mod test {
    use super::PostResolver;
    use crate::{
        account::AccountService, attachment::AttachmentService, custom_emoji::CustomEmojiService,
        job::JobService,
    };
    use athena::JobQueue;
    use core::convert::Infallible;
    use diesel::{QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use hyper::{Body, Request, Response};
    use iso8601_timestamp::Timestamp;
    use kitsune_activitypub::Fetcher;
    use kitsune_cache::NoopCache;
    use kitsune_config::instance::FederationFilterConfiguration;
    use kitsune_db::{
        model::{
            account::Account, custom_emoji::CustomEmoji, media_attachment::NewMediaAttachment,
        },
        schema::{accounts, custom_emojis, media_attachments},
    };
    use kitsune_federation_filter::FederationFilter;
    use kitsune_http_client::Client;
    use kitsune_jobs::KitsuneContextRepo;
    use kitsune_search::NoopSearchService;
    use kitsune_storage::fs::Storage as FsStorage;
    use kitsune_test::{build_ap_response, database_test, redis_test};
    use kitsune_url::UrlService;
    use kitsune_util::try_join;
    use kitsune_webfinger::Webfinger;
    use pretty_assertions::assert_eq;
    use scoped_futures::ScopedFutureExt;
    use speedy_uuid::Uuid;
    use std::sync::Arc;
    use tower::service_fn;

    #[tokio::test]
    #[serial_test::serial]
    async fn parse_post() {
        redis_test(|redis_pool| async move {
            database_test(|db_pool| async move {
                let post = "Hello @0x0@corteximplant.com! How are you doing? :blobhaj_happy: :blobhaj_sad@example.com:";

                let client = service_fn(|req: Request<_>| async move {
                    match req.uri().path_and_query().unwrap().as_str() {
                        "/.well-known/webfinger?resource=acct:0x0@corteximplant.com"=> {
                            let body = include_str!("../../../../test-fixtures/0x0_jrd.json");
                            Ok::<_, Infallible>(Response::new(Body::from(body)))
                        }
                        "/users/0x0" => {
                            let body = include_str!("../../../../test-fixtures/0x0_actor.json");
                            Ok::<_, Infallible>(build_ap_response(body))
                        }
                        path => panic!("HTTP client hit unexpected route: {path}"),
                    }
                });
                let client = Client::builder().service(client);

                let webfinger = Arc::new(Webfinger::with_client(client.clone(), Arc::new(NoopCache.into())));

                let fetcher = Fetcher::builder()
                    .client(client)
                    .db_pool(db_pool.clone())
                    .embed_client(None)
                    .federation_filter(
                        FederationFilter::new(&FederationFilterConfiguration::Deny {
                            domains: Vec::new(),
                        })
                        .unwrap(),
                    )
                    .resolver(webfinger.clone())
                    .search_backend(NoopSearchService)
                    .account_cache(Arc::new(NoopCache.into()))
                    .post_cache(Arc::new(NoopCache.into()))
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
                    .media_proxy_enabled(true)
                    .storage_backend(FsStorage::new("uploads".into()))
                    .url_service(url_service.clone())
                    .build();

                let account_service = AccountService::builder()
                    .attachment_service(attachment_service.clone())
                    .db_pool(db_pool.clone())
                    .fetcher(fetcher)
                    .job_service(job_service)
                    .resolver(webfinger)
                    .url_service(url_service.clone())
                    .build();

                let custom_emoji_service = CustomEmojiService::builder()
                    .attachment_service(attachment_service.clone())
                    .db_pool(db_pool.clone())
                    .url_service(url_service.clone())
                    .build();

                let emoji_ids = (Uuid::now_v7(), Uuid::now_v7());
                let media_attachment_ids = (Uuid::now_v7(), Uuid::now_v7());
                db_pool
                    .with_connection(|db_conn| {
                        async {
                            let media_fut = diesel::insert_into(media_attachments::table)
                                .values(NewMediaAttachment {
                                    id: media_attachment_ids.0,
                                    content_type: "image/jpeg",
                                    account_id: None,
                                    description: None,
                                    blurhash: None,
                                    file_path: None,
                                    remote_url: None,
                                })
                                .execute(db_conn);
                            let emoji_fut = diesel::insert_into(custom_emojis::table)
                                .values(CustomEmoji {
                                    id: emoji_ids.0,
                                    shortcode: String::from("blobhaj_happy"),
                                    domain: None,
                                    remote_id: String::from("https://local.domain/emoji/blobhaj_happy"),
                                    media_attachment_id: media_attachment_ids.0,
                                    endorsed: false,
                                    created_at: Timestamp::now_utc(),
                                    updated_at: Timestamp::now_utc()
                                })
                                .execute(db_conn);
                            try_join!(media_fut, emoji_fut)
                        }.scoped()
                    })
                    .await
                    .expect("Failed to insert the local emoji");

                db_pool
                    .with_connection(|db_conn| {
                        async {
                            let media_fut = diesel::insert_into(media_attachments::table)
                                .values(NewMediaAttachment {
                                    id: media_attachment_ids.1,
                                    content_type: "image/jpeg",
                                    account_id: None,
                                    description: None,
                                    blurhash: None,
                                    file_path: None,
                                    remote_url: Some("https://media.example.com/emojis/blobhaj.jpeg"),
                                })
                                .execute(db_conn);
                            let emoji_fut = diesel::insert_into(custom_emojis::table)
                                .values(CustomEmoji {
                                    id: emoji_ids.1,
                                    shortcode: String::from("blobhaj_sad"),
                                    domain: Some(String::from("example.com")),
                                    remote_id: String::from("https://example.com/emojis/1"),
                                    media_attachment_id: media_attachment_ids.1,
                                    endorsed: false,
                                    created_at: Timestamp::now_utc(),
                                    updated_at: Timestamp::now_utc(),
                                })
                                .execute(db_conn);
                            try_join!(media_fut, emoji_fut)
                        }.scoped()
                    })
                    .await
                    .expect("Failed to insert the remote emoji");

                let post_resolver = PostResolver::builder()
                    .account(account_service)
                    .custom_emoji(custom_emoji_service)
                    .build();

                let resolved = post_resolver
                    .resolve(post)
                    .await
                    .expect("Failed to resolve the post");

                assert_eq!(resolved.content, "Hello <a class=\"mention\" href=\"https://corteximplant.com/users/0x0\">@0x0@corteximplant.com</a>! How are you doing? :blobhaj_happy: :blobhaj_sad__example_com:");
                assert_eq!(resolved.mentioned_accounts.len(), 1);
                assert_eq!(resolved.custom_emojis.len(), 2);

                let (account_id, _mention_text) = &resolved.mentioned_accounts[0];
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

                assert_eq!(resolved.custom_emojis[0], (emoji_ids.1, String::from(":blobhaj_sad__example_com:")));
                assert_eq!(resolved.custom_emojis[1], (emoji_ids.0, String::from(":blobhaj_happy:")));
            }).await;
        }).await;
    }
}
