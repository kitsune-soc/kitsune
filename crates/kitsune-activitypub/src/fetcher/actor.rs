use super::Fetcher;
use crate::process_attachments;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_cache::CacheBackend;
use kitsune_core::traits::fetcher::AccountFetchOptions;
use kitsune_db::{
    changeset::UpdateAccount,
    insert::{
        NewAccount, NewAccountsActivitypub, NewAccountsCryptographicKey, NewCryptographicKey,
    },
    model::Account,
    schema::{accounts, accounts_activitypub, accounts_cryptographic_keys, cryptographic_keys},
    with_connection, with_transaction,
};
use kitsune_error::{Error, Result, kitsune_error};
use kitsune_search::SearchBackend;
use kitsune_type::ap::actor::Actor;
use kitsune_util::{convert::timestamp_to_uuid, sanitize::CleanHtmlExt};
use url::Url;

impl Fetcher {
    /// Fetch an ActivityPub actor
    ///
    /// # Panics
    ///
    /// - Panics if the URL doesn't contain a host section
    #[cfg_attr(not(coverage), instrument(skip(self)))]
    #[allow(clippy::too_many_lines)]
    pub(crate) async fn fetch_actor(
        &self,
        opts: AccountFetchOptions<'_>,
    ) -> Result<Option<Account>> {
        // Obviously we can't hit the cache nor the database if we wanna refetch the actor
        if !opts.refetch {
            if let Some(user) = self.account_cache.get(opts.url).await? {
                return Ok(Some(user));
            }

            let user_data = with_connection!(self.db_pool, |db_conn| {
                accounts::table
                    .filter(accounts::url.eq(opts.url))
                    .select(Account::as_select())
                    .first(db_conn)
                    .await
                    .optional()
            })?;

            if let Some(user) = user_data {
                return Ok(Some(user));
            }
        }

        let mut url = Url::parse(opts.url)?;
        let Some(mut actor) = self.fetch_ap_resource::<_, Actor>(url.clone()).await? else {
            return Ok(None);
        };

        let mut domain = url
            .host_str()
            .ok_or_else(|| kitsune_error!("missing host component"))?;

        let domain_buf;
        let try_resolver = opts
            .acct
            .is_none_or(|acct| acct != (&actor.preferred_username, domain));

        let used_resolver = if try_resolver {
            match self
                .resolver
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

        if !used_resolver && actor.id != url.as_str() {
            url = Url::parse(&actor.id)?;
            domain = url
                .host_str()
                .ok_or_else(|| kitsune_error!("missing host component"))?;
        }

        actor.clean_html();

        let account: Account = with_transaction!(self.db_pool, |tx| {
            let account = diesel::insert_into(accounts::table)
                .values(NewAccount {
                    id: timestamp_to_uuid(actor.published),
                    account_type: actor.r#type.into(),
                    avatar_id: None,
                    header_id: None,
                    display_name: actor.name.as_deref(),
                    note: actor.subject.as_deref(),
                    username: actor.preferred_username.as_str(),
                    locked: actor.manually_approves_followers,
                    local: false,
                    domain,
                    url: actor.id.as_str(),
                    created_at: Some(actor.published),
                })
                .on_conflict(accounts::url)
                .do_update()
                .set(UpdateAccount {
                    display_name: actor.name.as_deref(),
                    note: actor.subject.as_deref(),
                    locked: Some(actor.manually_approves_followers),
                    ..Default::default()
                })
                .returning(Account::as_select())
                .get_result::<Account>(tx)
                .await?;

            // Insert or update cryptographic key
            diesel::insert_into(cryptographic_keys::table)
                .values(NewCryptographicKey {
                    key_id: actor.public_key.id.as_str(),
                    public_key_der: &[], // TODO: Parse PEM to DER
                    private_key_der: None,
                })
                .on_conflict(cryptographic_keys::key_id)
                .do_nothing()
                .execute(tx)
                .await?;

            // Insert ActivityPub data
            diesel::insert_into(accounts_activitypub::table)
                .values(NewAccountsActivitypub {
                    account_id: account.id,
                    featured_collection_url: actor.featured.as_deref(),
                    followers_url: actor.followers.as_deref(),
                    following_url: actor.following.as_deref(),
                    inbox_url: Some(actor.inbox.as_str()),
                    outbox_url: actor.outbox.as_deref(),
                    shared_inbox_url: actor
                        .endpoints
                        .as_ref()
                        .and_then(|endpoints| endpoints.shared_inbox.as_deref()),
                    key_id: actor.public_key.id.as_str(),
                })
                .on_conflict(accounts_activitypub::account_id)
                .do_update()
                .set((
                    accounts_activitypub::featured_collection_url.eq(actor.featured.as_deref()),
                    accounts_activitypub::followers_url.eq(actor.followers.as_deref()),
                    accounts_activitypub::following_url.eq(actor.following.as_deref()),
                    accounts_activitypub::inbox_url.eq(Some(actor.inbox.as_str())),
                    accounts_activitypub::outbox_url.eq(actor.outbox.as_deref()),
                    accounts_activitypub::shared_inbox_url.eq(actor
                        .endpoints
                        .as_ref()
                        .and_then(|endpoints| endpoints.shared_inbox.as_deref())),
                    accounts_activitypub::key_id.eq(actor.public_key.id.as_str()),
                ))
                .execute(tx)
                .await?;

            // Link account to cryptographic key
            diesel::insert_into(accounts_cryptographic_keys::table)
                .values(NewAccountsCryptographicKey {
                    account_id: account.id,
                    key_id: actor.public_key.id.as_str(),
                })
                .on_conflict_do_nothing()
                .execute(tx)
                .await?;

            let avatar_id = match actor.icon {
                Some(icon) => process_attachments(tx, &account, &[icon]).await?.pop(),
                _ => None,
            };

            let header_id = match actor.image {
                Some(image) => process_attachments(tx, &account, &[image]).await?.pop(),
                _ => None,
            };

            let mut update_changeset = UpdateAccount::default();
            if let Some(avatar_id) = avatar_id {
                update_changeset.avatar_id = Some(avatar_id);
            }

            if let Some(header_id) = header_id {
                update_changeset.header_id = Some(header_id);
            }

            let account = if avatar_id.is_some() || header_id.is_some() {
                diesel::update(&account)
                    .set(update_changeset)
                    .returning(Account::as_select())
                    .get_result(tx)
                    .await?
            } else {
                account
            };

            Ok::<_, Error>(account)
        })?;

        self.search_backend
            .add_to_index(account.clone().into())
            .await?;

        Ok(Some(account))
    }
}
