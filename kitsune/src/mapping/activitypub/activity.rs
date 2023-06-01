use super::IntoObject;
use crate::{error::Result, state::Zustand, try_join};
use async_trait::async_trait;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{account::Account, favourite::Favourite, follower::Follow, post::Post},
    schema::{accounts, posts},
};
use kitsune_type::ap::{ap_context, helper::StringOrObject, Activity, ActivityType, ObjectField};
use time::OffsetDateTime;

#[async_trait]
pub trait IntoActivity {
    type Output;
    type NegateOutput;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output>;
    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput>;
}

#[async_trait]
impl IntoActivity for Account {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output> {
        let account_url = state.service.url.user_url(self.id);

        Ok(Activity {
            context: ap_context(),
            id: format!("{account_url}#update"),
            r#type: ActivityType::Update,
            actor: StringOrObject::String(account_url),
            object: ObjectField::Actor(self.into_object(state).await?),
            published: OffsetDateTime::now_utc(),
        })
    }

    async fn into_negate_activity(self, _state: &Zustand) -> Result<Self::NegateOutput> {
        todo!();
    }
}

#[async_trait]
impl IntoActivity for Favourite {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output> {
        let mut db_conn = state.db_conn.get().await?;
        let account_url_fut = accounts::table
            .find(self.account_id)
            .select(accounts::url)
            .get_result::<String>(&mut db_conn);

        let post_url_fut = posts::table
            .find(self.post_id)
            .select(posts::url)
            .get_result(&mut db_conn);

        let (account_url, post_url) = try_join!(account_url_fut, post_url_fut)?;

        Ok(Activity {
            context: ap_context(),
            id: self.url,
            r#type: ActivityType::Like,
            actor: StringOrObject::String(account_url),
            object: ObjectField::Url(post_url),
            published: self.created_at,
        })
    }

    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput> {
        let mut db_conn = state.db_conn.get().await?;
        let account_url = accounts::table
            .find(self.account_id)
            .select(accounts::url)
            .get_result::<String>(&mut db_conn)
            .await?;

        Ok(Activity {
            context: ap_context(),
            id: format!("{}#undo", self.url),
            r#type: ActivityType::Undo,
            actor: StringOrObject::String(account_url.clone()),
            object: ObjectField::Activity(self.into_activity(state).await?.into()),
            published: OffsetDateTime::now_utc(),
        })
    }
}

#[async_trait]
impl IntoActivity for Follow {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output> {
        let mut db_conn = state.db_conn.get().await?;
        let attributed_to_fut = accounts::table
            .find(self.follower_id)
            .select(accounts::url)
            .get_result::<String>(&mut db_conn);

        let object_fut = accounts::table
            .find(self.account_id)
            .select(accounts::url)
            .get_result::<String>(&mut db_conn);

        let (attributed_to, object) = try_join!(attributed_to_fut, object_fut)?;

        Ok(Activity {
            context: ap_context(),
            id: self.url,
            actor: StringOrObject::String(attributed_to),
            r#type: ActivityType::Follow,
            object: ObjectField::Url(object),
            published: self.created_at,
        })
    }

    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput> {
        let mut db_conn = state.db_conn.get().await?;
        let attributed_to = accounts::table
            .find(self.follower_id)
            .select(accounts::url)
            .get_result::<String>(&mut db_conn)
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

#[async_trait]
impl IntoActivity for Post {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output> {
        let account_url = state.service.url.user_url(self.account_id);

        if let Some(reposted_post_id) = self.reposted_post_id {
            let mut db_conn = state.db_conn.get().await?;
            let reposted_post_url = posts::table
                .find(reposted_post_id)
                .select(posts::url)
                .get_result(&mut db_conn)
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

    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput> {
        let account_url = state.service.url.user_url(self.account_id);

        let activity = if self.reposted_post_id.is_some() {
            Activity {
                context: ap_context(),
                id: format!("{}#undo", self.url),
                r#type: ActivityType::Undo,
                actor: StringOrObject::String(account_url),
                object: ObjectField::Url(self.url),
                published: OffsetDateTime::now_utc(),
            }
        } else {
            let object = self.into_object(state).await?;

            Activity {
                context: ap_context(),
                id: format!("{}#delete", object.id),
                r#type: ActivityType::Delete,
                actor: StringOrObject::String(account_url),
                published: OffsetDateTime::now_utc(),
                object: ObjectField::Object(object),
            }
        };

        Ok(activity)
    }
}
