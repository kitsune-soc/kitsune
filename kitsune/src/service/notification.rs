use std::cmp::min;

use crate::{
    consts::{API_DEFAULT_LIMIT, API_MAX_LIMIT},
    error::{Error, Result},
};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use futures_util::{Stream, TryStreamExt};
use kitsune_db::{
    model::notification::{Notification, NotificationType},
    schema::notifications,
    PgPool,
};
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct NotificationService {
    db_pool: PgPool,
}

#[derive(Clone, TypedBuilder)]
pub struct GetNotifications {
    /// ID of the account whose notifications are getting fetched
    account_id: Uuid,

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
                    .eq(get_notifications.account_id)
                    .and(notifications::notification_type.eq_any(get_notifications.included_types))
                    .and(notifications::notification_type.ne_all(get_notifications.excluded_types)),
            )
            .select(Notification::as_select())
            .order(notifications::id.desc())
            .limit(min(get_notifications.limit, API_MAX_LIMIT) as i64)
            .into_boxed();

        if let Some(account_id) = get_notifications.triggering_account_id {
            query = query.filter(notifications::triggering_account_id.eq(account_id));
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
            .with_connection(|mut db_conn| async move {
                Ok::<_, Error>(query.load_stream(&mut db_conn).await?.map_err(Error::from))
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
            .with_connection(|mut db_conn| async move {
                notifications::table
                    .filter(
                        notifications::id
                            .eq(id)
                            .and(notifications::receiving_account_id.eq(account_id)),
                    )
                    .select(Notification::as_select())
                    .get_result(&mut db_conn)
                    .await
                    .optional()
            })
            .await
            .map_err(Error::from)
    }

    pub async fn dismiss(&self, id: Uuid, account_id: Uuid) -> Result<()> {
        self.db_pool
            .with_connection(|mut db_conn| async move {
                diesel::delete(
                    notifications::table.filter(
                        notifications::id
                            .eq(id)
                            .and(notifications::receiving_account_id.eq(account_id)),
                    ),
                )
                .execute(&mut db_conn)
                .await
            })
            .await?;

        Ok(())
    }

    pub async fn clear_all(&self, account_id: Uuid) -> Result<()> {
        self.db_pool
            .with_connection(|mut db_conn| async move {
                diesel::delete(
                    notifications::table.filter(notifications::receiving_account_id.eq(account_id)),
                )
                .execute(&mut db_conn)
                .await
            })
            .await?;

        Ok(())
    }
}
