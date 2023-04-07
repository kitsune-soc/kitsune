use super::IntoObject;
use crate::{error::Result, state::Zustand, util::BaseToCc};
use async_trait::async_trait;
use chrono::Utc;
use kitsune_db::{
    column::UrlQuery,
    entity::{
        accounts, accounts_followers, favourites, posts,
        prelude::{Accounts, Posts},
    },
    link::FavouritedPostAuthor,
};
use kitsune_type::ap::{
    ap_context, helper::StringOrObject, Activity, ActivityType, BaseObject, PUBLIC_IDENTIFIER,
};
use sea_orm::{EntityTrait, ModelTrait, QuerySelect};

#[async_trait]
pub trait IntoActivity {
    type Output;
    type NegateOutput;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output>;
    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput>;
}

#[async_trait]
impl IntoActivity for favourites::Model {
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

        let author_account_url = self
            .find_linked(FavouritedPostAuthor)
            .select_only()
            .column(accounts::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Post without related account");

        let post_url = self
            .find_related(Posts)
            .select_only()
            .column(posts::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Favourite without associated post");

        Ok(Activity {
            actor: account_url.clone(),
            r#type: ActivityType::Like,
            object: StringOrObject::String(post_url),
            rest: BaseObject {
                context: ap_context(),
                id: self.url,
                attributed_to: Some(StringOrObject::String(account_url)),
                in_reply_to: None,
                sensitive: false,
                published: self.created_at.into(),
                to: vec![author_account_url, PUBLIC_IDENTIFIER.to_string()],
                cc: vec![],
            },
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

        let author_account_url = self
            .find_linked(FavouritedPostAuthor)
            .select_only()
            .column(accounts::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Post without related account");

        Ok(Activity {
            actor: account_url.clone(),
            r#type: ActivityType::Undo,
            rest: BaseObject {
                context: ap_context(),
                id: format!("{}#undo", self.url),
                attributed_to: Some(StringOrObject::String(account_url)),
                in_reply_to: None,
                sensitive: false,
                published: Utc::now(),
                to: vec![author_account_url, PUBLIC_IDENTIFIER.to_string()],
                cc: vec![],
            },
            object: StringOrObject::String(self.url),
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
            actor: attributed_to.clone(),
            r#type: ActivityType::Follow,
            object: StringOrObject::String(object.clone()),
            rest: BaseObject {
                context: ap_context(),
                id: self.url,
                attributed_to: Some(StringOrObject::String(attributed_to)),
                in_reply_to: None,
                sensitive: false,
                published: self.created_at.into(),
                to: vec![object],
                cc: vec![],
            },
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

        let followed = Accounts::find_by_id(self.account_id)
            .select_only()
            .column(accounts::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Follow without followed");

        Ok(Activity {
            actor: attributed_to.clone(),
            r#type: ActivityType::Undo,
            rest: BaseObject {
                context: ap_context(),
                id: format!("{}#undo", self.url),
                attributed_to: Some(StringOrObject::String(attributed_to)),
                in_reply_to: None,
                sensitive: false,
                published: self.created_at.into(),
                to: vec![followed],
                cc: vec![],
            },
            object: StringOrObject::String(self.url),
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
            let (to, cc) = self.visibility.base_to_cc(&account);

            Ok(Activity {
                actor: account.url.clone(),
                r#type: ActivityType::Announce,
                object: StringOrObject::String(reposted_post_url),
                rest: BaseObject {
                    context: ap_context(),
                    id: format!("{}/activity", self.url),
                    attributed_to: Some(StringOrObject::String(account.url)),
                    in_reply_to: None,
                    sensitive: false,
                    published: self.created_at.into(),
                    to,
                    cc,
                },
            })
        } else {
            let created_at = self.created_at;
            let object = self.into_object(state).await?;

            Ok(Activity {
                actor: account.url.clone(),
                r#type: ActivityType::Create,
                rest: BaseObject {
                    context: ap_context(),
                    id: format!("{}/activity", object.rest.id),
                    attributed_to: Some(StringOrObject::String(account.url)),
                    in_reply_to: None,
                    sensitive: false,
                    published: created_at.into(),
                    to: object.rest.to.clone(),
                    cc: object.rest.cc.clone(),
                },
                object: StringOrObject::Object(object),
            })
        }
    }

    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput> {
        let account = Accounts::find_by_id(self.account_id)
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Post without author");

        // TODO: Decide type by `reposted_post_id` field
        let object = self.into_object(state).await?;

        Ok(Activity {
            actor: account.url.clone(),
            r#type: ActivityType::Delete,
            rest: BaseObject {
                context: ap_context(),
                id: format!("{}#delete", object.rest.id),
                sensitive: false,
                attributed_to: None,
                in_reply_to: None,
                published: Utc::now(),
                to: object.rest.to,
                cc: object.rest.cc,
            },
            object: StringOrObject::String(object.rest.id),
        })
    }
}
