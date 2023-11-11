use super::IntoObject;
use crate::{error::Result, state::State};
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    model::{account::Account, favourite::Favourite, follower::Follow, post::Post},
    schema::{accounts, posts},
};
use kitsune_type::ap::{ap_context, helper::StringOrObject, Activity, ActivityType, ObjectField};
use kitsune_util::try_join;
use scoped_futures::ScopedFutureExt;
use std::future::Future;

pub trait IntoActivity {
    type Output;
    type NegateOutput;

    fn into_activity(self, state: &State) -> impl Future<Output = Result<Self::Output>> + Send;
    fn into_negate_activity(
        self,
        state: &State,
    ) -> impl Future<Output = Result<Self::NegateOutput>> + Send;
}

impl IntoActivity for Account {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &State) -> Result<Self::Output> {
        let account_url = state.service.url.user_url(self.id);

        Ok(Activity {
            context: ap_context(),
            id: format!("{account_url}#update"),
            r#type: ActivityType::Update,
            actor: StringOrObject::String(account_url),
            object: ObjectField::Actor(self.into_object(state).await?),
            published: Timestamp::now_utc(),
        })
    }

    async fn into_negate_activity(self, _state: &State) -> Result<Self::NegateOutput> {
        todo!();
    }
}

impl IntoActivity for Favourite {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &State) -> Result<Self::Output> {
        let (account_url, post_url) = state
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    let account_url_fut = accounts::table
                        .find(self.account_id)
                        .select(accounts::url)
                        .get_result::<String>(db_conn);

                    let post_url_fut = posts::table
                        .find(self.post_id)
                        .select(posts::url)
                        .get_result(db_conn);

                    try_join!(account_url_fut, post_url_fut)
                }
                .scoped()
            })
            .await?;

        Ok(Activity {
            context: ap_context(),
            id: self.url,
            r#type: ActivityType::Like,
            actor: StringOrObject::String(account_url),
            object: ObjectField::Url(post_url),
            published: self.created_at,
        })
    }

    async fn into_negate_activity(self, state: &State) -> Result<Self::NegateOutput> {
        let account_url = state
            .db_pool
            .with_connection(|db_conn| {
                accounts::table
                    .find(self.account_id)
                    .select(accounts::url)
                    .get_result::<String>(db_conn)
                    .scoped()
            })
            .await?;

        Ok(Activity {
            context: ap_context(),
            id: format!("{}#undo", self.url),
            r#type: ActivityType::Undo,
            actor: StringOrObject::String(account_url.clone()),
            object: ObjectField::Activity(self.into_activity(state).await?.into()),
            published: Timestamp::now_utc(),
        })
    }
}

impl IntoActivity for Follow {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &State) -> Result<Self::Output> {
        let (attributed_to, object) = state
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    let attributed_to_fut = accounts::table
                        .find(self.follower_id)
                        .select(accounts::url)
                        .get_result::<String>(db_conn);

                    let object_fut = accounts::table
                        .find(self.account_id)
                        .select(accounts::url)
                        .get_result::<String>(db_conn);

                    try_join!(attributed_to_fut, object_fut)
                }
                .scoped()
            })
            .await?;

        Ok(Activity {
            context: ap_context(),
            id: self.url,
            actor: StringOrObject::String(attributed_to),
            r#type: ActivityType::Follow,
            object: ObjectField::Url(object),
            published: self.created_at,
        })
    }

    async fn into_negate_activity(self, state: &State) -> Result<Self::NegateOutput> {
        let attributed_to = state
            .db_pool
            .with_connection(|db_conn| {
                accounts::table
                    .find(self.follower_id)
                    .select(accounts::url)
                    .get_result::<String>(db_conn)
                    .scoped()
            })
            .await?;

        Ok(Activity {
            context: ap_context(),
            id: format!("{}#undo", self.url),
            r#type: ActivityType::Undo,
            actor: StringOrObject::String(attributed_to),
            published: self.created_at,
            object: ObjectField::Activity(self.into_activity(state).await?.into()),
        })
    }
}

impl IntoActivity for Post {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &State) -> Result<Self::Output> {
        let account_url = state.service.url.user_url(self.account_id);

        if let Some(reposted_post_id) = self.reposted_post_id {
            let reposted_post_url = state
                .db_pool
                .with_connection(|db_conn| {
                    posts::table
                        .find(reposted_post_id)
                        .select(posts::url)
                        .get_result(db_conn)
                        .scoped()
                })
                .await?;

            Ok(Activity {
                context: ap_context(),
                id: format!("{}/activity", self.url),
                r#type: ActivityType::Announce,
                actor: StringOrObject::String(account_url),
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
                actor: StringOrObject::String(account_url),
                published: created_at,
                object: ObjectField::Object(object),
            })
        }
    }

    async fn into_negate_activity(self, state: &State) -> Result<Self::NegateOutput> {
        let account_url = state.service.url.user_url(self.account_id);

        let activity = if self.reposted_post_id.is_some() {
            Activity {
                context: ap_context(),
                id: format!("{}#undo", self.url),
                r#type: ActivityType::Undo,
                actor: StringOrObject::String(account_url),
                object: ObjectField::Url(self.url),
                published: Timestamp::now_utc(),
            }
        } else {
            let object = self.into_object(state).await?;

            Activity {
                context: ap_context(),
                id: format!("{}#delete", object.id),
                r#type: ActivityType::Delete,
                actor: StringOrObject::String(account_url),
                published: Timestamp::now_utc(),
                object: ObjectField::Object(object),
            }
        };

        Ok(activity)
    }
}
