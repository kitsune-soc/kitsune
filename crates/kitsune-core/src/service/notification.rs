use crate::{
    consts::{API_DEFAULT_LIMIT, API_MAX_LIMIT},
    error::{Error, Result},
};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl,
    SelectableHelper,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures_util::{Stream, TryStreamExt};
use kitsune_db::{
    model::notification::{NewNotification, Notification, NotificationType},
    schema::{accounts, accounts_follows, accounts_preferences, notifications, posts},
    PgPool,
};
use scoped_futures::ScopedFutureExt;
use speedy_uuid::Uuid;
use std::cmp::min;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct NotificationService {
    db_pool: PgPool,
}

#[derive(Clone, TypedBuilder)]
pub struct GetNotifications {
    /// ID of the account whose notifications are getting fetched
    receiving_account_id: Uuid,

    /// ID of the account which triggered the notifications
    #[builder(default)]
    triggering_account_id: Option<Uuid>,

    /// Included notification types
    included_types: Vec<NotificationType>,

    /// excluded notification types
    excluded_types: Vec<NotificationType>,

    /// Limit of returned posts
    #[builder(default = API_DEFAULT_LIMIT)]
    limit: usize,

    /// Smallest ID, return results starting from this ID
    ///
    /// Used for pagination
    #[builder(default)]
    min_id: Option<Uuid>,

    /// Smallest ID, return highest results
    ///
    /// Used for pagination
    #[builder(default)]
    since_id: Option<Uuid>,

    /// Largest ID
    ///
    /// Used for pagination
    #[builder(default)]
    max_id: Option<Uuid>,
}

impl NotificationService {
    pub async fn get_notifications(
        &self,
        get_notifications: GetNotifications,
    ) -> Result<impl Stream<Item = Result<Notification>> + '_> {
        let mut query = notifications::table
            .filter(
                notifications::receiving_account_id
                    .eq(get_notifications.receiving_account_id)
                    .and(notifications::notification_type.ne_all(get_notifications.excluded_types)),
            )
            .select(Notification::as_select())
            .order(notifications::id.desc())
            .limit(min(get_notifications.limit, API_MAX_LIMIT) as i64)
            .into_boxed();

        if let Some(account_id) = get_notifications.triggering_account_id {
            query = query.filter(notifications::triggering_account_id.eq(account_id));
        }
        if !get_notifications.included_types.is_empty() {
            query = query
                .filter(notifications::notification_type.eq_any(get_notifications.included_types));
        }
        if let Some(since_id) = get_notifications.since_id {
            query = query.filter(notifications::id.gt(since_id));
        }
        if let Some(max_id) = get_notifications.max_id {
            query = query.filter(notifications::id.lt(max_id));
        }
        if let Some(min_id) = get_notifications.min_id {
            query = query
                .filter(notifications::id.gt(min_id))
                .order(notifications::id.asc());
        }

        self.db_pool
            .with_connection(|mut db_conn| {
                async move {
                    Ok::<_, Error>(query.load_stream(&mut db_conn).await?.map_err(Error::from))
                }
                .scoped()
            })
            .await
            .map_err(Error::from)
    }

    pub async fn get_notification_by_id(
        &self,
        id: Uuid,
        account_id: Uuid,
    ) -> Result<Option<Notification>> {
        self.db_pool
            .with_connection(|mut db_conn| {
                async move {
                    notifications::table
                        .filter(
                            notifications::id
                                .eq(id)
                                .and(notifications::receiving_account_id.eq(account_id)),
                        )
                        .select(Notification::as_select())
                        .first(&mut db_conn)
                        .await
                        .optional()
                }
                .scoped()
            })
            .await
            .map_err(Error::from)
    }

    pub async fn dismiss(&self, id: Uuid, account_id: Uuid) -> Result<()> {
        self.db_pool
            .with_connection(|mut db_conn| {
                async move {
                    diesel::delete(
                        notifications::table.filter(
                            notifications::id
                                .eq(id)
                                .and(notifications::receiving_account_id.eq(account_id)),
                        ),
                    )
                    .execute(&mut db_conn)
                    .await
                }
                .scoped()
            })
            .await?;

        Ok(())
    }

    pub async fn clear_all(&self, account_id: Uuid) -> Result<()> {
        self.db_pool
            .with_connection(|mut db_conn| {
                async move {
                    diesel::delete(
                        notifications::table
                            .filter(notifications::receiving_account_id.eq(account_id)),
                    )
                    .execute(&mut db_conn)
                    .await
                }
                .scoped()
            })
            .await?;

        Ok(())
    }

    pub async fn notify_on_new_post(
        tx: &mut AsyncPgConnection,
        author_id: Uuid,
        post_id: Uuid,
    ) -> Result<()> {
        let accounts_to_notify: Vec<Uuid> = accounts::table
            .inner_join(accounts_follows::table.on(accounts::id.eq(accounts_follows::follower_id)))
            .filter(
                accounts_follows::account_id
                    .eq(author_id)
                    .and(accounts_follows::notify.eq(true))
                    .and(accounts::local.eq(true)),
            )
            .select(accounts_follows::follower_id)
            .load_stream::<Uuid>(tx)
            .await?
            .try_collect()
            .await?;

        diesel::insert_into(notifications::table)
            .values(
                accounts_to_notify
                    .iter()
                    .map(|acc| {
                        NewNotification::builder()
                            .receiving_account_id(*acc)
                            .post(author_id, post_id)
                    })
                    .collect::<Vec<Notification>>(),
            )
            .on_conflict_do_nothing()
            .execute(tx)
            .await?;

        Ok(())
    }

    pub async fn notify_on_update_post(
        tx: &mut AsyncPgConnection,
        author_id: Uuid,
        post_id: Uuid,
    ) -> Result<()> {
        let accounts_to_notify: Vec<Uuid> = posts::table
            .inner_join(
                accounts_preferences::table
                    .on(posts::account_id.eq(accounts_preferences::account_id)),
            )
            .inner_join(accounts::table)
            .filter(
                posts::reposted_post_id
                    .eq(post_id)
                    .and(accounts_preferences::notify_on_post_update)
                    .and(accounts::local.eq(true)),
            )
            .select(accounts_preferences::account_id)
            .load_stream::<Uuid>(tx)
            .await?
            .try_collect()
            .await?;

        diesel::insert_into(notifications::table)
            .values(
                accounts_to_notify
                    .iter()
                    .map(|acc| {
                        NewNotification::builder()
                            .receiving_account_id(*acc)
                            .post_update(author_id, post_id)
                    })
                    .collect::<Vec<Notification>>(),
            )
            .on_conflict_do_nothing()
            .execute(tx)
            .await?;

        Ok(())
    }

    pub async fn notify_on_repost(
        tx: &mut AsyncPgConnection,
        notified_id: Uuid,
        reposter: Uuid,
        post_id: Uuid,
    ) -> Result<()> {
        let notified_account_id = accounts::table
            .inner_join(accounts_preferences::table)
            .filter(
                accounts::id
                    .eq(notified_id)
                    .and(accounts_preferences::notify_on_repost.eq(true)),
            )
            .select(accounts::id)
            .get_result::<Uuid>(tx)
            .await
            .optional()?;

        if let Some(account_id) = notified_account_id {
            diesel::insert_into(notifications::table)
                .values(
                    NewNotification::builder()
                        .receiving_account_id(account_id)
                        .repost(reposter, post_id),
                )
                .on_conflict_do_nothing()
                .execute(tx)
                .await?;
        }

        Ok(())
    }
}
