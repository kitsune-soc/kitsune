use super::process_attachments;
use crate::{
    activitypub::{process_new_object, ProcessNewObject},
    consts::USER_AGENT,
    error::{ApiError, Error, Result},
    sanitize::CleanHtmlExt,
    service::federation_filter::FederationFilterService,
    util::timestamp_to_uuid,
};
use async_recursion::async_recursion;
use autometrics::autometrics;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
use http::HeaderValue;
use kitsune_cache::{ArcCache, CacheBackend};
use kitsune_db::{
    model::{
        account::{Account, AccountConflictChangeset, NewAccount, UpdateAccountMedia},
        post::Post,
    },
    schema::{accounts, posts},
    PgPool,
};
use kitsune_embed::Client as EmbedClient;
use kitsune_http_client::Client;
use kitsune_search::{SearchBackend, SearchService};
use kitsune_type::ap::{actor::Actor, Object};
use typed_builder::TypedBuilder;
use url::Url;

// Maximum call depth of fetching new posts. Prevents unbounded recursion.
// Setting this to >=40 would cause the `fetch_infinitely_long_reply_chain` test to run into stack overflow
const MAX_FETCH_DEPTH: u32 = 30;

#[derive(Clone, Debug, TypedBuilder)]
/// Options passed to the fetcher
pub struct FetchOptions<'a> {
    /// Refetch the ActivityPub entity
    ///
    /// This is mainly used to refresh possibly stale actors
    ///
    /// Default: false
    #[builder(default = false)]
    refetch: bool,

    /// URL of the ActivityPub entity
    url: &'a str,
}

impl<'a> From<&'a str> for FetchOptions<'a> {
    fn from(value: &'a str) -> Self {
        Self::builder().url(value).build()
    }
}

#[derive(Clone, TypedBuilder)]
pub struct Fetcher {
    #[builder(default =
        Client::builder()
            .default_header(
                "accept",
                HeaderValue::from_static(
                    "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\", application/activity+json",
                ),
            )
            .unwrap()
            .user_agent(USER_AGENT)
            .unwrap()
            .build()
    )]
    client: Client,
    db_pool: PgPool,
    embed_client: Option<EmbedClient>,
    federation_filter: FederationFilterService,
    #[builder(setter(into))]
    search_service: SearchService,

    // Caches
    post_cache: ArcCache<str, Post>,
    user_cache: ArcCache<str, Account>,
}

impl Fetcher {
    /// Fetch an ActivityPub actor
    ///
    /// # Panics
    ///
    /// - Panics if the URL doesn't contain a host section
    #[instrument(skip(self))]
    #[autometrics(track_concurrency)]
    pub async fn fetch_actor(&self, opts: FetchOptions<'_>) -> Result<Account> {
        // Obviously we can't hit the cache nor the database if we wanna refetch the actor
        if !opts.refetch {
            if let Some(user) = self.user_cache.get(opts.url).await? {
                return Ok(user);
            }

            let user_data = self
                .db_pool
                .with_connection(|mut db_conn| async move {
                    accounts::table
                        .filter(accounts::url.eq(opts.url))
                        .select(Account::as_select())
                        .first(&mut db_conn)
                        .await
                        .optional()
                })
                .await?;

            if let Some(user) = user_data {
                return Ok(user);
            }
        }

        let url = Url::parse(opts.url)?;
        if !self.federation_filter.is_url_allowed(&url)? {
            return Err(ApiError::Unauthorised.into());
        }

        let mut actor: Actor = self.client.get(url.as_str()).await?.json().await?;
        actor.clean_html();

        let account: Account = self
            .db_pool
            .with_transaction(|tx| {
                async move {
                    let account = diesel::insert_into(accounts::table)
                        .values(NewAccount {
                            id: timestamp_to_uuid(actor.published),
                            display_name: actor.name.as_deref(),
                            note: actor.subject.as_deref(),
                            username: actor.preferred_username.as_str(),
                            locked: actor.manually_approves_followers,
                            local: false,
                            domain: url.host_str().unwrap(),
                            actor_type: actor.r#type.into(),
                            url: actor.id.as_str(),
                            featured_collection_url: actor.featured.as_deref(),
                            followers_url: actor.followers.as_deref(),
                            following_url: actor.following.as_deref(),
                            inbox_url: Some(actor.inbox.as_str()),
                            outbox_url: actor.outbox.as_deref(),
                            shared_inbox_url: actor
                                .endpoints
                                .and_then(|endpoints| endpoints.shared_inbox)
                                .as_deref(),
                            public_key_id: actor.public_key.id.as_str(),
                            public_key: actor.public_key.public_key_pem.as_str(),
                            created_at: Some(actor.published),
                        })
                        .on_conflict(accounts::url)
                        .do_update()
                        .set(AccountConflictChangeset {
                            display_name: actor.name.as_deref(),
                            note: actor.subject.as_deref(),
                            locked: actor.manually_approves_followers,
                            public_key_id: actor.public_key.id.as_str(),
                            public_key: actor.public_key.public_key_pem.as_str(),
                        })
                        .returning(Account::as_returning())
                        .get_result::<Account>(tx)
                        .await?;

                    let avatar_id = if let Some(icon) = actor.icon {
                        process_attachments(tx, &account, &[icon]).await?.pop()
                    } else {
                        None
                    };

                    let header_id = if let Some(image) = actor.image {
                        process_attachments(tx, &account, &[image]).await?.pop()
                    } else {
                        None
                    };

                    let mut update_changeset = UpdateAccountMedia::default();
                    if let Some(avatar_id) = avatar_id {
                        update_changeset = UpdateAccountMedia {
                            avatar_id: Some(avatar_id),
                            ..update_changeset
                        };
                    }
                    if let Some(header_id) = header_id {
                        update_changeset = UpdateAccountMedia {
                            header_id: Some(header_id),
                            ..update_changeset
                        };
                    }

                    Ok::<_, Error>(match update_changeset {
                        UpdateAccountMedia {
                            avatar_id: None,
                            header_id: None,
                        } => account,
                        _ => {
                            diesel::update(&account)
                                .set(update_changeset)
                                .returning(Account::as_returning())
                                .get_result(tx)
                                .await?
                        }
                    })
                }
                .scope_boxed()
            })
            .await?;

        self.search_service
            .add_to_index(account.clone().into())
            .await?;

        Ok(account)
    }

    #[async_recursion]
    pub(super) async fn fetch_object_inner(
        &self,
        url: &str,
        call_depth: u32,
    ) -> Result<Option<Post>> {
        if call_depth > MAX_FETCH_DEPTH {
            return Ok(None);
        }

        if !self.federation_filter.is_url_allowed(&Url::parse(url)?)? {
            return Err(ApiError::Unauthorised.into());
        }

        if let Some(post) = self.post_cache.get(url).await? {
            return Ok(Some(post));
        }

        let post = self
            .db_pool
            .with_connection(|mut db_conn| async move {
                posts::table
                    .filter(posts::url.eq(url))
                    .select(Post::as_select())
                    .first(&mut db_conn)
                    .await
                    .optional()
            })
            .await?;

        if let Some(post) = post {
            self.post_cache.set(url, &post).await?;
            return Ok(Some(post));
        }

        let url = Url::parse(url)?;
        let object: Object = self.client.get(url.as_str()).await?.json().await?;

        let process_data = ProcessNewObject::builder()
            .call_depth(call_depth)
            .db_pool(&self.db_pool)
            .embed_client(self.embed_client.as_ref())
            .fetcher(self)
            .object(object)
            .search_service(&self.search_service)
            .build();
        let post = process_new_object(process_data).await?;

        self.post_cache.set(&post.url, &post).await?;

        Ok(Some(post))
    }

    #[instrument(skip(self))]
    #[autometrics(track_concurrency)]
    pub async fn fetch_object(&self, url: &str) -> Result<Post> {
        self.fetch_object_inner(url, 0)
            .await
            .transpose()
            .expect("[Bug] Highest level fetch returned a `None`")
    }
}

#[cfg(test)]
mod test {
    use super::MAX_FETCH_DEPTH;
    use crate::{
        activitypub::Fetcher,
        config::FederationFilterConfiguration,
        error::{ApiError, Error},
        service::federation_filter::FederationFilterService,
        test::database_test,
    };
    use core::convert::Infallible;
    use diesel::{QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use hyper::{Body, Request, Response, Uri};
    use iso8601_timestamp::Timestamp;
    use kitsune_cache::NoopCache;
    use kitsune_db::{model::account::Account, schema::accounts};
    use kitsune_http_client::Client;
    use kitsune_search::NoopSearchService;
    use kitsune_type::ap::{
        actor::{Actor, ActorType, PublicKey},
        ap_context, AttributedToField, Object, ObjectType, PUBLIC_IDENTIFIER,
    };
    use pretty_assertions::assert_eq;
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };
    use tower::service_fn;

    #[tokio::test]
    #[serial_test::serial]
    async fn fetch_actor() {
        database_test(|db_pool| async move {
            let client = Client::builder().service(service_fn(handle));

            let fetcher = Fetcher::builder()
                .client(client)
                .db_pool(db_pool)
                .embed_client(None)
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

            let user = fetcher
                .fetch_actor("https://corteximplant.com/users/0x0".into())
                .await
                .expect("Fetch actor");

            assert_eq!(user.username, "0x0");
            assert_eq!(user.domain, "corteximplant.com");
            assert_eq!(user.url, "https://corteximplant.com/users/0x0");
            assert_eq!(
                user.inbox_url,
                Some("https://corteximplant.com/users/0x0/inbox".into())
            );
        })
        .await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn fetch_note() {
        database_test(|db_pool| async move {
            let client = Client::builder().service(service_fn(handle));

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
                .post_cache(Arc::new(NoopCache.into()))
                .user_cache(Arc::new(NoopCache.into()))
                .build();

            let note = fetcher
                .fetch_object("https://corteximplant.com/@0x0/109501674056556919")
                .await
                .expect("Fetch note");
            assert_eq!(
                note.url,
                "https://corteximplant.com/users/0x0/statuses/109501674056556919"
            );

            let author = db_pool
                .with_connection(|mut db_conn| {
                    accounts::table
                        .find(note.account_id)
                        .select(Account::as_select())
                        .get_result::<Account>(&mut db_conn)
                })
                .await
                .expect("Get author");

            assert_eq!(author.username, "0x0");
            assert_eq!(author.url, "https://corteximplant.com/users/0x0");
        })
        .await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn fetch_infinitely_long_reply_chain() {
        database_test(|db_pool| async move {
            let request_counter = Arc::new(AtomicU32::new(0));
            let client = service_fn(move |req: Request<_>| {
                let count = request_counter.fetch_add(1, Ordering::SeqCst);
                assert!(MAX_FETCH_DEPTH * 3 >= count);
                async move {
                    let author_id = "https://example.com/users/1".to_owned();
                    let author = Actor {
                        context: ap_context(),
                        id: author_id.clone(),
                        r#type: ActorType::Person,
                        name: None,
                        preferred_username: "Infinite notes".into(),
                        subject: None,
                        icon: None,
                        image: None,
                        manually_approves_followers: false,
                        public_key: PublicKey {
                            id: format!("{author_id}#main-key"),
                            owner: author_id,
                            // A 512-bit RSA public key generated as a placeholder
                            public_key_pem: "-----BEGIN PUBLIC KEY-----\nMFwwDQYJKoZIhvcNAQEBBQADSwAwSAJBAK1v4oRbdBPi8oRL0M1GQqSWtkb9uE2L\nJCAgZK9KiVECNYvEASYor7DeMEu6BxR1E4XI2DlGkigClWXFhQDhos0CAwEAAQ==\n-----END PUBLIC KEY-----\n".into(),
                        },
                        endpoints: None,
                        featured: None,
                        inbox: "https://example.com/inbox".into(),
                        outbox: None,
                        followers: None,
                        following: None,
                        published: Timestamp::UNIX_EPOCH,
                    };

                    if let Some(note_id) = req.uri().path_and_query().unwrap().as_str().strip_prefix("/notes/") {
                        let note_id = note_id.parse::<u32>().unwrap();
                        let note = Object {
                            context: ap_context(),
                            id: format!("https://example.com/notes/{note_id}"),
                            r#type: ObjectType::Note,
                            attributed_to: AttributedToField::Url(author.id.clone()),
                            in_reply_to: Some(format!("https://example.com/notes/{}", note_id + 1)),
                            name: None,
                            summary: None,
                            content: String::new(),
                            media_type: None,
                            attachment: Vec::new(),
                            tag: Vec::new(),
                            sensitive: false,
                            published: Timestamp::UNIX_EPOCH,
                            to: vec![PUBLIC_IDENTIFIER.into()],
                            cc: Vec::new(),
                        };
                        let body = simd_json::to_string(&note).unwrap();
                        Ok::<_, Infallible>(Response::new(Body::from(body)))
                    } else {
                        assert_eq!(
                            req.uri().path_and_query().unwrap(),
                            Uri::try_from(&author.id).unwrap().path_and_query().unwrap()
                        );
                        let body = simd_json::to_string(&author).unwrap();
                        Ok::<_, Infallible>(Response::new(Body::from(body)))
                    }
                }
            });
            let client = Client::builder().service(client);

            let fetcher = Fetcher::builder()
                .client(client)
                .db_pool(db_pool)
                .embed_client(None)
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

            assert!(fetcher
                .fetch_object("https://example.com/notes/0")
                .await
                .is_ok());
        })
        .await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn federation_allow() {
        database_test(|db_pool| async move {
            let builder = Fetcher::builder()
                .db_pool(db_pool)
                .embed_client(None)
                .federation_filter(
                    FederationFilterService::new(&FederationFilterConfiguration::Allow {
                        domains: vec!["corteximplant.com".into()],
                    })
                    .unwrap(),
                )
                .search_service(NoopSearchService)
                .post_cache(Arc::new(NoopCache.into()))
                .user_cache(Arc::new(NoopCache.into()));

            let client = service_fn(
                #[allow(unreachable_code)] // https://github.com/rust-lang/rust/issues/67227
                |_: Request<_>| async {
                    panic!("Requested a denied domain") as Result<Response<Body>, Infallible>
                },
            );
            let client = Client::builder().service(client);
            let fetcher = builder.clone().client(client).build();

            assert!(matches!(
                fetcher.fetch_object("https://example.com/fakeobject").await,
                Err(Error::Api(ApiError::Unauthorised))
            ));
            assert!(matches!(
                fetcher
                    .fetch_object("https://other.badstuff.com/otherfake")
                    .await,
                Err(Error::Api(ApiError::Unauthorised))
            ));

            let client = Client::builder().service(service_fn(handle));
            let fetcher = builder.client(client).build();

            assert!(matches!(
                fetcher
                    .fetch_object("https://corteximplant.com/@0x0/109501674056556919")
                    .await,
                Ok(..)
            ));
        })
        .await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn federation_deny() {
        database_test(|db_pool| async move {
            let client = service_fn(
                #[allow(unreachable_code)]
                |_: Request<_>| async {
                    panic!("Requested a denied domain") as Result<Response<Body>, Infallible>
                },
            );
            let client = Client::builder().service(client);

            let fetcher = Fetcher::builder()
                .client(client)
                .db_pool(db_pool)
                .embed_client(None)
                .federation_filter(
                    FederationFilterService::new(&FederationFilterConfiguration::Deny {
                        domains: vec!["example.com".into(), "*.badstuff.com".into()],
                    })
                    .unwrap(),
                )
                .search_service(NoopSearchService)
                .post_cache(Arc::new(NoopCache.into()))
                .user_cache(Arc::new(NoopCache.into()))
                .build();

            assert!(matches!(
                fetcher.fetch_object("https://example.com/fakeobject").await,
                Err(Error::Api(ApiError::Unauthorised))
            ));
            assert!(matches!(
                fetcher
                    .fetch_object("https://other.badstuff.com/otherfake")
                    .await,
                Err(Error::Api(ApiError::Unauthorised))
            ));
        })
        .await;
    }

    async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        match req.uri().path_and_query().unwrap().as_str() {
            "/users/0x0" => {
                let body = include_str!("../test-fixtures/0x0_actor.json");
                Ok::<_, Infallible>(Response::new(Body::from(body)))
            }
            "/@0x0/109501674056556919" => {
                let body =
                    include_str!("../test-fixtures/corteximplant.com_109501674056556919.json");
                Ok::<_, Infallible>(Response::new(Body::from(body)))
            }
            "/users/0x0/statuses/109501659207519785" => {
                let body =
                    include_str!("../test-fixtures/corteximplant.com_109501659207519785.json");
                Ok::<_, Infallible>(Response::new(Body::from(body)))
            }
            path => panic!("HTTP client hit unexpected route: {path}"),
        }
    }
}
