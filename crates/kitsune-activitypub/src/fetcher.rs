use super::process_attachments;
use crate::{
    activitypub::{process_new_object, ProcessNewObject},
    consts::USER_AGENT,
    error::{ApiError, Error, Result},
    service::federation_filter::FederationFilterService,
    util::timestamp_to_uuid,
    webfinger::Webfinger,
};
use async_recursion::async_recursion;
use autometrics::autometrics;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use headers::{ContentType, HeaderMapExt};
use http::HeaderValue;
use iso8601_timestamp::Timestamp;
use kitsune_cache::{ArcCache, CacheBackend};
use kitsune_db::{
    model::{
        account::{Account, AccountConflictChangeset, NewAccount, UpdateAccountMedia},
        custom_emoji::CustomEmoji,
        media_attachment::{MediaAttachment, NewMediaAttachment},
        post::Post,
    },
    schema::{accounts, custom_emojis, media_attachments, posts},
    PgPool,
};
use kitsune_embed::Client as EmbedClient;
use kitsune_http_client::Client;
use kitsune_search::SearchBackend;
use kitsune_type::{
    ap::{actor::Actor, emoji::Emoji, Object},
    jsonld::RdfNode,
};
use kitsune_util::sanitize::CleanHtmlExt;
use mime::Mime;
use scoped_futures::ScopedFutureExt;
use serde::de::DeserializeOwned;
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;
use url::Url;

// Maximum call depth of fetching new posts. Prevents unbounded recursion.
// Setting this to >=40 would cause the `fetch_infinitely_long_reply_chain` test to run into stack overflow
const MAX_FETCH_DEPTH: u32 = 30;

#[derive(Clone, Debug, TypedBuilder)]
/// Options passed to the fetcher
pub struct FetchOptions<'a> {
    /// Prefetched WebFinger `acct` URI
    #[builder(default, setter(strip_option))]
    acct: Option<(&'a str, &'a str)>,

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
    search_backend: kitsune_search::AnySearchBackend,
    webfinger: Webfinger,

    // Caches
    post_cache: ArcCache<str, Post>,
    user_cache: ArcCache<str, Account>,
}

impl Fetcher {
    async fn fetch_ap_resource<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned + RdfNode,
    {
        let response = self.client.get(url).await?;
        let Some(content_type) = response
            .headers()
            .typed_get::<ContentType>()
            .map(Mime::from)
        else {
            return Err(ApiError::BadRequest.into());
        };

        let is_json_ld_activitystreams = || {
            content_type
                .essence_str()
                .eq_ignore_ascii_case("application/ld+json")
                && content_type
                    .get_param("profile")
                    .map_or(false, |profile_urls| {
                        profile_urls
                            .as_str()
                            .split_whitespace()
                            .any(|url| url == "https://www.w3.org/ns/activitystreams")
                    })
        };

        let is_activity_json = || {
            content_type
                .essence_str()
                .eq_ignore_ascii_case("application/activity+json")
        };

        if !is_json_ld_activitystreams() && !is_activity_json() {
            return Err(ApiError::BadRequest.into());
        }

        Ok(response.jsonld().await?)
    }

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
                .with_connection(|db_conn| {
                    async move {
                        accounts::table
                            .filter(accounts::url.eq(opts.url))
                            .select(Account::as_select())
                            .first(db_conn)
                            .await
                            .optional()
                    }
                    .scoped()
                })
                .await?;

            if let Some(user) = user_data {
                return Ok(user);
            }
        }

        let mut url = Url::parse(opts.url)?;
        if !self.federation_filter.is_url_allowed(&url)? {
            return Err(ApiError::Unauthorised.into());
        }

        let mut actor: Actor = self.fetch_ap_resource(url.as_str()).await?;

        let mut domain = url.host_str().ok_or(ApiError::MissingHost)?;
        let domain_buf;
        let fetch_webfinger = opts
            .acct
            .map_or(true, |acct| acct != (&actor.preferred_username, domain));

        let used_webfinger = if fetch_webfinger {
            match self
                .webfinger
                .resolve_actor(&actor.preferred_username, domain)
                .await?
            {
                Some(resource) if resource.uri == actor.id => {
                    actor.preferred_username = resource.username;
                    domain_buf = resource.domain;
                    domain = &domain_buf;
                    true
                }
                _ => {
                    // Fall back to `{preferredUsername}@{domain}`
                    false
                }
            }
        } else {
            false
        };
        if !used_webfinger && actor.id != url.as_str() {
            url = Url::parse(&actor.id)?;
            domain = url.host_str().ok_or(ApiError::MissingHost)?;
        }

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
                            domain,
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

        self.search_backend
            .add_to_index(account.clone().into())
            .await?;

        Ok(account)
    }

    pub async fn fetch_emoji(&self, url: &str) -> Result<CustomEmoji> {
        let existing_emoji = self
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    custom_emojis::table
                        .filter(custom_emojis::remote_id.eq(url))
                        .select(CustomEmoji::as_select())
                        .first(db_conn)
                        .await
                        .optional()
                }
                .scoped()
            })
            .await?;

        if let Some(emoji) = existing_emoji {
            return Ok(emoji);
        }

        let mut url = Url::parse(url)?;
        if !self.federation_filter.is_url_allowed(&url)? {
            return Err(ApiError::Unauthorised.into());
        }

        let emoji: Emoji = self.client.get(url.as_str()).await?.jsonld().await?;

        let mut domain = url.host_str().ok_or(ApiError::MissingHost)?;

        if emoji.id != url.as_str() {
            url = Url::parse(&emoji.id)?;
            domain = url.host_str().ok_or(ApiError::MissingHost)?;
        }

        let content_type = emoji
            .icon
            .media_type
            .as_deref()
            .or_else(|| mime_guess::from_path(&emoji.icon.url).first_raw())
            .ok_or(ApiError::UnsupportedMediaType)?;

        let name_pure = emoji.name.replace(':', "");

        let emoji: CustomEmoji = self
            .db_pool
            .with_transaction(|tx| {
                async move {
                    let media_attachment = diesel::insert_into(media_attachments::table)
                        .values(NewMediaAttachment {
                            id: Uuid::now_v7(),
                            account_id: None,
                            content_type,
                            description: None,
                            blurhash: None,
                            file_path: None,
                            remote_url: Some(&emoji.icon.url),
                        })
                        .returning(MediaAttachment::as_returning())
                        .get_result::<MediaAttachment>(tx)
                        .await?;
                    let emoji = diesel::insert_into(custom_emojis::table)
                        .values(CustomEmoji {
                            id: Uuid::now_v7(),
                            remote_id: emoji.id,
                            shortcode: name_pure.to_string(),
                            domain: Some(domain.to_string()),
                            media_attachment_id: media_attachment.id,
                            endorsed: false,
                            created_at: Timestamp::now_utc(),
                            updated_at: Timestamp::now_utc(),
                        })
                        .returning(CustomEmoji::as_returning())
                        .get_result::<CustomEmoji>(tx)
                        .await?;
                    Ok::<_, Error>(emoji)
                }
                .scope_boxed()
            })
            .await?;
        Ok(emoji)
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
            .with_connection(|db_conn| {
                async move {
                    posts::table
                        .filter(posts::url.eq(url))
                        .select(Post::as_select())
                        .first(db_conn)
                        .await
                        .optional()
                }
                .scoped()
            })
            .await?;

        if let Some(post) = post {
            self.post_cache.set(url, &post).await?;
            return Ok(Some(post));
        }

        let url = Url::parse(url)?;
        let object: Object = self.fetch_ap_resource(url.as_str()).await?;

        let process_data = ProcessNewObject::builder()
            .call_depth(call_depth)
            .db_pool(&self.db_pool)
            .embed_client(self.embed_client.as_ref())
            .fetcher(self)
            .object(Box::new(object))
            .search_backend(&self.search_backend)
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
        error::{ApiError, Error},
        service::federation_filter::FederationFilterService,
        webfinger::Webfinger,
    };
    use diesel::{QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use http::{header::CONTENT_TYPE, uri::PathAndQuery};
    use hyper::{Body, Request, Response, StatusCode, Uri};
    use iso8601_timestamp::Timestamp;
    use kitsune_cache::NoopCache;
    use kitsune_config::instance::FederationFilterConfiguration;
    use kitsune_db::{
        model::{account::Account, media_attachment::MediaAttachment},
        schema::{accounts, media_attachments},
    };
    use kitsune_http_client::Client;
    use kitsune_search::NoopSearchService;
    use kitsune_test::{build_ap_response, database_test};
    use kitsune_type::{
        ap::{
            actor::{Actor, ActorType, PublicKey},
            ap_context, AttributedToField, Object, ObjectType, PUBLIC_IDENTIFIER,
        },
        webfinger::{Link, Resource},
    };
    use pretty_assertions::assert_eq;
    use scoped_futures::ScopedFutureExt;
    use std::{
        convert::Infallible,
        sync::{
            atomic::{AtomicU32, Ordering},
            Arc,
        },
    };
    use tower::service_fn;

    #[tokio::test]
    #[serial_test::serial]
    async fn fetch_actor() {
        database_test(|db_pool| async move {
            let client = Client::builder().service(service_fn(handle));

            let fetcher = Fetcher::builder()
                .client(client.clone())
                .db_pool(db_pool)
                .embed_client(None)
                .federation_filter(
                    FederationFilterService::new(&FederationFilterConfiguration::Deny {
                        domains: Vec::new(),
                    })
                    .unwrap(),
                )
                .search_backend(NoopSearchService)
                .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
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
    async fn fetch_actor_with_custom_acct() {
        database_test(|db_pool| async move {
            let mut jrd_base = include_bytes!("../../../../test-fixtures/0x0_jrd.json").to_owned();
            let jrd_body = simd_json::to_string(&Resource {
                subject: "acct:0x0@joinkitsune.org".into(),
                ..simd_json::from_slice(&mut jrd_base).unwrap()
            })
            .unwrap();
            let client = service_fn(move |req: Request<_>| {
                let jrd_body = jrd_body.clone();
                async move {
                    match (
                        req.uri().authority().unwrap().as_str(),
                        req.uri().path_and_query().unwrap().as_str(),
                    ) {
                        (
                            "corteximplant.com",
                            "/.well-known/webfinger?resource=acct:0x0@corteximplant.com",
                        )
                        | (
                            "joinkitsune.org",
                            "/.well-known/webfinger?resource=acct:0x0@joinkitsune.org",
                        ) => Ok::<_, Infallible>(Response::new(Body::from(jrd_body))),
                        _ => handle(req).await,
                    }
                }
            });
            let client = Client::builder().service(client);

            let fetcher = Fetcher::builder()
                .client(client.clone())
                .db_pool(db_pool)
                .embed_client(None)
                .federation_filter(
                    FederationFilterService::new(&FederationFilterConfiguration::Deny {
                        domains: Vec::new(),
                    })
                    .unwrap(),
                )
                .search_backend(NoopSearchService)
                .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
                .post_cache(Arc::new(NoopCache.into()))
                .user_cache(Arc::new(NoopCache.into()))
                .build();

            let user = fetcher
                .fetch_actor("https://corteximplant.com/users/0x0".into())
                .await
                .expect("Fetch actor");

            assert_eq!(user.username, "0x0");
            assert_eq!(user.domain, "joinkitsune.org");
            assert_eq!(user.url, "https://corteximplant.com/users/0x0");
        })
        .await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn ignore_fake_webfinger_acct() {
        database_test(|db_pool| async move {
            let link = Link {
                rel: "self".to_owned(),
                r#type: Some("application/activity+json".to_owned()),
                href: Some("https://social.whitehouse.gov/users/POTUS".to_owned()),
            };
            let jrd = Resource {
                subject: "acct:POTUS@whitehouse.gov".into(),
                aliases: Vec::new(),
                links: vec![link.clone()],
            };
            let client = service_fn(move |req: Request<_>| {
                let link = link.clone();
                let jrd = jrd.clone();
                async move {
                    match (
                        req.uri().authority().unwrap().as_str(),
                        req.uri().path_and_query().unwrap().as_str(),
                    ) {
                        (
                            "corteximplant.com",
                            "/.well-known/webfinger?resource=acct:0x0@corteximplant.com",
                        ) => {
                            let fake_jrd = Resource {
                                links: vec![Link {
                                    href: Some("https://corteximplant.com/users/0x0".to_owned()),
                                    ..link
                                }],
                                ..jrd
                            };
                            let body = simd_json::to_string(&fake_jrd).unwrap();
                            Ok::<_, Infallible>(Response::new(Body::from(body)))
                        }
                        (
                            "whitehouse.gov",
                            "/.well-known/webfinger?resource=acct:POTUS@whitehouse.gov",
                        ) => {
                            let body = simd_json::to_string(&jrd).unwrap();
                            Ok(Response::new(Body::from(body)))
                        }
                        _ => handle(req).await,
                    }
                }
            });
            let client = Client::builder().service(client);

            let fetcher = Fetcher::builder()
                .client(client.clone())
                .db_pool(db_pool)
                .embed_client(None)
                .federation_filter(
                    FederationFilterService::new(&FederationFilterConfiguration::Deny {
                        domains: Vec::new(),
                    })
                    .unwrap(),
                )
                .search_backend(NoopSearchService)
                .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
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
        })
        .await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn fetch_note() {
        database_test(|db_pool| async move {
            let client = Client::builder().service(service_fn(handle));

            let fetcher = Fetcher::builder()
                .client(client.clone())
                .db_pool(db_pool.clone())
                .embed_client(None)
                .federation_filter(
                    FederationFilterService::new(&FederationFilterConfiguration::Deny {
                        domains: Vec::new(),
                    })
                    .unwrap(),
                )
                .search_backend(NoopSearchService)
                .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
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
                .with_connection(|db_conn| {
                    accounts::table
                        .find(note.account_id)
                        .select(Account::as_select())
                        .get_result::<Account>(db_conn)
                        .scoped()
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
                        preferred_username: "InfiniteNotes".into(),
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

                        Ok::<_, Infallible>(build_ap_response(body))
                    } else if req.uri().path_and_query().unwrap() == Uri::try_from(&author.id).unwrap().path_and_query().unwrap() {
                        let body = simd_json::to_string(&author).unwrap();

                        Ok::<_, Infallible>(build_ap_response(body))
                    } else {
                        handle(req).await
                    }
                }
            });
            let client = Client::builder().service(client);

            let fetcher = Fetcher::builder()
                .client(client.clone())
                .db_pool(db_pool)
                .embed_client(None)
                .federation_filter(
                    FederationFilterService::new(&FederationFilterConfiguration::Deny {
                        domains: Vec::new(),
                    })
                    .unwrap(),
                )
                .search_backend(NoopSearchService)
                .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
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
    async fn check_ap_id_authority() {
        database_test(|db_pool| async move {
            let builder = Fetcher::builder()
                .db_pool(db_pool)
                .embed_client(None)
                .federation_filter(
                    FederationFilterService::new(&FederationFilterConfiguration::Deny {
                        domains: Vec::new(),
                    })
                    .unwrap(),
                )
                .search_backend(NoopSearchService)
                .post_cache(Arc::new(NoopCache.into()))
                .user_cache(Arc::new(NoopCache.into()));

            let client = service_fn(|req: Request<_>| {
                assert_ne!(req.uri().host(), Some("corteximplant.com"));
                handle(req)
            });
            let client = Client::builder().service(client);
            let fetcher = builder
                .clone()
                .client(client.clone())
                .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
                .build();

            // The mock HTTP client ensures that the fetcher doesn't access the correct server
            // so this should return error
            let _ = fetcher
                .fetch_actor("https://example.com/users/0x0".into())
                .await
                .unwrap_err();

            let client = service_fn(|req: Request<_>| {
                // Let `fetch_object` fetch `attributedTo`
                if req.uri().path_and_query().map(PathAndQuery::as_str) != Some("/users/0x0") {
                    assert_ne!(req.uri().host(), Some("corteximplant.com"));
                }

                handle(req)
            });
            let client = Client::builder().service(client);
            let fetcher = builder
                .clone()
                .client(client.clone())
                .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
                .build();

            let _ = fetcher
                .fetch_object("https://example.com/@0x0/109501674056556919")
                .await
                .unwrap_err();
        })
        .await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn check_ap_content_type() {
        database_test(|db_pool| async move {
            let client = service_fn(|req: Request<_>| async {
                let mut res = handle(req).await.unwrap();
                res.headers_mut().remove(CONTENT_TYPE);
                Ok::<_, Infallible>(res)
            });
            let client = Client::builder().service(client);

            let fetcher = Fetcher::builder()
                .client(client.clone())
                .db_pool(db_pool)
                .embed_client(None)
                .federation_filter(
                    FederationFilterService::new(&FederationFilterConfiguration::Deny {
                        domains: Vec::new(),
                    })
                    .unwrap(),
                )
                .search_backend(NoopSearchService)
                .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
                .post_cache(Arc::new(NoopCache.into()))
                .user_cache(Arc::new(NoopCache.into()))
                .build();

            assert!(matches!(
                fetcher
                    .fetch_object("https://corteximplant.com/users/0x0")
                    .await,
                Err(Error::Api(ApiError::BadRequest))
            ));
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
                .search_backend(NoopSearchService)
                .post_cache(Arc::new(NoopCache.into()))
                .user_cache(Arc::new(NoopCache.into()));

            let client = service_fn(
                #[allow(unreachable_code)] // https://github.com/rust-lang/rust/issues/67227
                |_: Request<_>| async {
                    panic!("Requested a denied domain") as Result<Response<Body>, Infallible>
                },
            );
            let client = Client::builder().service(client);
            let fetcher = builder
                .clone()
                .client(client.clone())
                .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
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

            let client = Client::builder().service(service_fn(handle));
            let fetcher = builder
                .clone()
                .client(client.clone())
                .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
                .build();

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
                .client(client.clone())
                .db_pool(db_pool)
                .embed_client(None)
                .federation_filter(
                    FederationFilterService::new(&FederationFilterConfiguration::Deny {
                        domains: vec!["example.com".into(), "*.badstuff.com".into()],
                    })
                    .unwrap(),
                )
                .search_backend(NoopSearchService)
                .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
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

    #[tokio::test]
    #[serial_test::serial]
    async fn fetch_emoji() {
        database_test(|db_pool| async move {
            let client = Client::builder().service(service_fn(handle));

            let fetcher = Fetcher::builder()
                .client(client.clone())
                .db_pool(db_pool.clone())
                .embed_client(None)
                .federation_filter(
                    FederationFilterService::new(&FederationFilterConfiguration::Deny {
                        domains: Vec::new(),
                    })
                    .unwrap(),
                )
                .search_backend(NoopSearchService)
                .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
                .post_cache(Arc::new(NoopCache.into()))
                .user_cache(Arc::new(NoopCache.into()))
                .build();

            let emoji = fetcher
                .fetch_emoji("https://corteximplant.com/emojis/7952")
                .await
                .expect("Fetch emoji");
            assert_eq!(emoji.shortcode, "Blobhaj");
            assert_eq!(emoji.domain, Some(String::from("corteximplant.com")));

            let media_attachment = db_pool
                .with_connection(|db_conn| {
                    media_attachments::table
                        .find(emoji.media_attachment_id)
                        .select(MediaAttachment::as_select())
                        .get_result::<MediaAttachment>(db_conn)
                        .scoped()
                })
                .await
                .expect("Get media attachment");

            assert_eq!(
                media_attachment.remote_url,
                Some(String::from(
                    "https://corteximplant.com/system/custom_emojis/images/000/007/952/original/33b7f12bd094b815.png"
                )));
        })
        .await;
    }

    async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        match req.uri().path_and_query().unwrap().as_str() {
            "/users/0x0" => {
                let body = include_str!("../../../../test-fixtures/0x0_actor.json");
                Ok::<_, Infallible>(build_ap_response(body))
            }
            "/@0x0/109501674056556919" => {
                let body = include_str!(
                    "../../../../test-fixtures/corteximplant.com_109501674056556919.json"
                );
                Ok::<_, Infallible>(build_ap_response(body))
            }
            "/users/0x0/statuses/109501659207519785" => {
                let body = include_str!(
                    "../../../../test-fixtures/corteximplant.com_109501659207519785.json"
                );
                Ok::<_, Infallible>(build_ap_response(body))
            }
            "/emojis/7952" => {
                let body =
                    include_str!("../../../../test-fixtures/corteximplant.com_emoji_7952.json");
                Ok::<_, Infallible>(build_ap_response(body))
            }
            "/emojis/8933" => {
                let body =
                    include_str!("../../../../test-fixtures/corteximplant.com_emoji_8933.json");
                Ok::<_, Infallible>(build_ap_response(body))
            }
            "/.well-known/webfinger?resource=acct:0x0@corteximplant.com" => {
                let body = include_str!("../../../../test-fixtures/0x0_jrd.json");
                Ok::<_, Infallible>(Response::new(Body::from(body)))
            }
            path if path.starts_with("/.well-known/webfinger?") => Ok::<_, Infallible>(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap(),
            ),
            path => panic!("HTTP client hit unexpected route: {path}"),
        }
    }
}
