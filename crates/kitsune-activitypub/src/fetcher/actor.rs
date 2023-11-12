use super::Fetcher;
use crate::{
    error::{Error, Result},
    process_attachments,
};
use autometrics::autometrics;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_cache::CacheBackend;
use kitsune_core::traits::{fetcher::AccountFetchOptions, Resolver};
use kitsune_db::{
    model::account::{Account, AccountConflictChangeset, NewAccount, UpdateAccountMedia},
    schema::accounts,
};
use kitsune_search::SearchBackend;
use kitsune_type::ap::actor::Actor;
use kitsune_util::{convert::timestamp_to_uuid, sanitize::CleanHtmlExt};
use scoped_futures::ScopedFutureExt;
use url::Url;

impl Fetcher {
    /// Fetch an ActivityPub actor
    ///
    /// # Panics
    ///
    /// - Panics if the URL doesn't contain a host section
    #[instrument(skip(self))]
    #[autometrics(track_concurrency)]
    pub async fn fetch_actor(&self, opts: AccountFetchOptions<'_>) -> Result<Account> {
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
        let mut actor: Actor = self.fetch_ap_resource(url.clone()).await?;

        let mut domain = url.host_str().ok_or(Error::MissingHost)?;
        let domain_buf;
        let fetch_webfinger = opts
            .acct
            .map_or(true, |acct| acct != (&actor.preferred_username, domain));

        let used_webfinger = if fetch_webfinger {
            match self
                .webfinger
                .resolve_account(&actor.preferred_username, domain)
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
            domain = url.host_str().ok_or(Error::MissingHost)?;
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

                    let account = match update_changeset {
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
                    };

                    Ok::<_, Error>(account)
                }
                .scope_boxed()
            })
            .await?;

        self.search_backend
            .add_to_index(account.clone().into())
            .await?;

        Ok(account)
    }
}
