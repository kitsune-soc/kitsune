use super::IntoObject;
use crate::{error::Result, state::Zustand};
use async_trait::async_trait;
use chrono::Utc;
use kitsune_db::{
    column::UrlQuery,
    entity::{
        accounts, accounts_followers, posts, posts_favourites,
        prelude::{Accounts, Posts},
    },
};
use kitsune_type::ap::{ap_context, helper::StringOrObject, Activity, ActivityType, ObjectField};
use sea_orm::{EntityTrait, ModelTrait, QuerySelect};

#[async_trait]
pub trait IntoActivity {
    type Output;
    type NegateOutput;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output>;
    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput>;
}

#[async_trait]
impl IntoActivity for accounts::Model {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output> {
        Ok(Activity {
            context: ap_context(),
            id: format!("{}#update", self.url),
            r#type: ActivityType::Update,
            actor: StringOrObject::String(self.url.clone()),
            object: ObjectField::Actor(self.into_object(state).await?),
            published: Utc::now(),
        })
    }

    async fn into_negate_activity(self, _state: &Zustand) -> Result<Self::NegateOutput> {
        todo!();
    }
}

#[async_trait]
impl IntoActivity for posts_favourites::Model {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output> {
        let account_url = self
            .find_related(Accounts)
            .select_only()
            .column(accounts::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Favourite without associated account");

        let post_url = self
            .find_related(Posts)
            .select_only()
            .column(posts::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Favourite without associated post");

        Ok(Activity {
            context: ap_context(),
            id: self.url,
            r#type: ActivityType::Like,
            actor: StringOrObject::String(account_url.clone()),
            object: ObjectField::Url(post_url),
            published: self.created_at.into(),
        })
    }

    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput> {
        let account_url = self
            .find_related(Accounts)
            .select_only()
            .column(accounts::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Favourite without associated account");

        Ok(Activity {
            context: ap_context(),
            id: format!("{}#undo", self.url),
            r#type: ActivityType::Undo,
            actor: StringOrObject::String(account_url.clone()),
            object: ObjectField::Activity(self.into_activity(state).await?.into()),
            published: Utc::now(),
        })
    }
}

#[async_trait]
impl IntoActivity for accounts_followers::Model {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output> {
        let attributed_to = Accounts::find_by_id(self.follower_id)
            .select_only()
            .column(accounts::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Follow without follower");

        let object = Accounts::find_by_id(self.account_id)
            .select_only()
            .column(accounts::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Follow without followed");

        Ok(Activity {
            context: ap_context(),
            id: self.url,
            actor: StringOrObject::String(attributed_to.clone()),
            r#type: ActivityType::Follow,
            object: ObjectField::Url(object),
            published: self.created_at.into(),
        })
    }

    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput> {
        let attributed_to = Accounts::find_by_id(self.follower_id)
            .select_only()
            .column(accounts::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Follow without follower");

        Ok(Activity {
            context: ap_context(),
            id: format!("{}#undo", self.url),
            r#type: ActivityType::Undo,
            actor: StringOrObject::String(attributed_to),
            published: self.created_at.into(),
            object: ObjectField::Activity(self.into_activity(state).await?.into()),
        })
    }
}

#[async_trait]
impl IntoActivity for posts::Model {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output> {
        let account = Accounts::find_by_id(self.account_id)
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Post without author");

        if let Some(reposted_post_id) = self.reposted_post_id {
            let reposted_post_url = Posts::find_by_id(reposted_post_id)
                .select_only()
                .column(posts::Column::Url)
                .into_values::<String, UrlQuery>()
                .one(&state.db_conn)
                .await?
                .expect("[Bug] Repost without associated post");

            Ok(Activity {
                context: ap_context(),
                id: format!("{}/activity", self.url),
                r#type: ActivityType::Announce,
                actor: StringOrObject::String(account.url.clone()),
                object: ObjectField::Url(reposted_post_url),
                published: self.created_at.into(),
            })
        } else {
            let created_at = self.created_at;
            let object = self.into_object(state).await?;

            Ok(Activity {
                context: ap_context(),
                id: format!("{}/activity", object.id),
                r#type: ActivityType::Create,
                actor: StringOrObject::String(account.url.clone()),
                published: created_at.into(),
                object: ObjectField::Object(object),
            })
        }
    }

    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput> {
        let account = Accounts::find_by_id(self.account_id)
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Post without author");

        let activity = if self.reposted_post_id.is_some() {
            Activity {
                context: ap_context(),
                id: format!("{}#undo", self.url),
                r#type: ActivityType::Undo,
                actor: StringOrObject::String(account.url),
                object: ObjectField::Url(self.url),
                published: Utc::now(),
            }
        } else {
            let object = self.into_object(state).await?;

            Activity {
                context: ap_context(),
                id: format!("{}#delete", object.id),
                r#type: ActivityType::Delete,
                actor: StringOrObject::String(account.url.clone()),
                published: Utc::now(),
                object: ObjectField::Object(object),
            }
        };

        Ok(activity)
    }
}
