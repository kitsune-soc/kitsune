use crate::{
    consts::USER_AGENT,
    error::{ApiError, Error, Result},
    process_attachments, process_new_object,
    service::federation_filter::FederationFilterService,
    util::timestamp_to_uuid,
    webfinger::Webfinger,
    ProcessNewObject,
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
