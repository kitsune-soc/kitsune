use super::{IntoObject, State};
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    model::{account::Account, favourite::Favourite, follower::Follow, post::Post},
    schema::{accounts, posts},
    with_connection,
};
use kitsune_error::Result;
use kitsune_type::ap::{Activity, ActivityType, ObjectField, ap_context};
use kitsune_util::try_join;

pub trait IntoActivity {
    type Output;
    type NegateOutput;

    fn into_activity(self, state: State<'_>) -> impl Future<Output = Result<Self::Output>> + Send;
    fn into_negate_activity(
        self,
        state: State<'_>,
    ) -> impl Future<Output = Result<Self::NegateOutput>> + Send;
}

impl IntoActivity for Account {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: State<'_>) -> Result<Self::Output> {
        let account_url = state.service.url.user_url(self.id);

        Ok(Activity {
            context: ap_context(),
            id: format!("{account_url}#update"),
            r#type: ActivityType::Update,
            actor: account_url,
            object: ObjectField::Actor(Box::new(self.into_object(state).await?)),
            published: Timestamp::now_utc(),
        })
    }

    async fn into_negate_activity(self, _state: State<'_>) -> Result<Self::NegateOutput> {
        todo!();
    }
}

impl IntoActivity for Favourite {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: State<'_>) -> Result<Self::Output> {
        let (account_url, post_url) = with_connection!(state.db_pool, |db_conn| {
            let account_url_fut = accounts::table
                .find(self.account_id)
                .select(accounts::url)
                .get_result::<String>(db_conn);

            let post_url_fut = posts::table
                .find(self.post_id)
                .select(posts::url)
                .get_result(db_conn);

            try_join!(account_url_fut, post_url_fut)
        })?;

        Ok(Activity {
            context: ap_context(),
            id: self.url,
            r#type: ActivityType::Like,
            actor: account_url,
            object: ObjectField::Url(post_url),
            published: self.created_at,
        })
    }

    async fn into_negate_activity(self, state: State<'_>) -> Result<Self::NegateOutput> {
        let account_url = with_connection!(state.db_pool, |db_conn| {
            accounts::table
                .find(self.account_id)
                .select(accounts::url)
                .get_result::<String>(db_conn)
                .await
        })?;

        Ok(Activity {
            context: ap_context(),
            id: format!("{}#undo", self.url),
            r#type: ActivityType::Undo,
            actor: account_url.clone(),
            object: ObjectField::Activity(self.into_activity(state).await?.into()),
            published: Timestamp::now_utc(),
        })
    }
}

impl IntoActivity for Follow {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: State<'_>) -> Result<Self::Output> {
        let (attributed_to, object) = with_connection!(state.db_pool, |db_conn| {
            let attributed_to_fut = accounts::table
                .find(self.follower_id)
                .select(accounts::url)
                .get_result::<String>(db_conn);

            let object_fut = accounts::table
                .find(self.account_id)
                .select(accounts::url)
                .get_result::<String>(db_conn);

            try_join!(attributed_to_fut, object_fut)
        })?;

        Ok(Activity {
            context: ap_context(),
            id: self.url,
            actor: attributed_to,
            r#type: ActivityType::Follow,
            object: ObjectField::Url(object),
            published: self.created_at,
        })
    }

    async fn into_negate_activity(self, state: State<'_>) -> Result<Self::NegateOutput> {
        let attributed_to = with_connection!(state.db_pool, |db_conn| {
            accounts::table
                .find(self.follower_id)
                .select(accounts::url)
                .get_result::<String>(db_conn)
                .await
        })?;

        Ok(Activity {
            context: ap_context(),
            id: format!("{}#undo", self.url),
            r#type: ActivityType::Undo,
            actor: attributed_to,
            published: self.created_at,
            object: ObjectField::Activity(self.into_activity(state).await?.into()),
        })
    }
}

impl IntoActivity for Post {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: State<'_>) -> Result<Self::Output> {
        let account_url = state.service.url.user_url(self.account_id);

        if let Some(reposted_post_id) = self.reposted_post_id {
            let reposted_post_url = with_connection!(state.db_pool, |db_conn| {
                posts::table
                    .find(reposted_post_id)
                    .select(posts::url)
                    .get_result(db_conn)
                    .await
            })?;

            Ok(Activity {
                context: ap_context(),
                id: format!("{}/activity", self.url),
                r#type: ActivityType::Announce,
                actor: account_url,
                object: ObjectField::Url(reposted_post_url),
                published: self.created_at,
            })
        } else {
            let created_at = self.created_at;
            let object = self.into_object(state).await?;

            Ok(Activity {
                context: ap_context(),
                id: format!("{}/activity", object.id),
                r#type: ActivityType::Create,
                actor: account_url,
                published: created_at,
                object: ObjectField::Object(Box::new(object)),
            })
        }
    }

    async fn into_negate_activity(self, state: State<'_>) -> Result<Self::NegateOutput> {
        let account_url = state.service.url.user_url(self.account_id);

        let activity = if self.reposted_post_id.is_some() {
            Activity {
                context: ap_context(),
                id: format!("{}#undo", self.url),
                r#type: ActivityType::Undo,
                actor: account_url,
                object: ObjectField::Url(self.url),
                published: Timestamp::now_utc(),
            }
        } else {
            let object = self.into_object(state).await?;

            Activity {
                context: ap_context(),
                id: format!("{}#delete", object.id),
                r#type: ActivityType::Delete,
                actor: account_url,
                published: Timestamp::now_utc(),
                object: ObjectField::Object(Box::new(object)),
            }
        };

        Ok(activity)
    }
}
