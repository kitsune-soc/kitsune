use super::IntoObject;
use crate::{
    db::{
        model::{account, favourite, post},
        UrlQuery,
    },
    error::Result,
    state::Zustand,
};
use async_trait::async_trait;
use chrono::Utc;
use phenomenon_type::ap::{
    ap_context, helper::StringOrObject, Activity, ActivityType, BaseObject, PUBLIC_IDENTIFIER,
};
use sea_orm::{ModelTrait, QuerySelect};

#[async_trait]
pub trait IntoActivity {
    type Output;
    type NegateOutput;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output>;
    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput>;
}

#[async_trait]
impl IntoActivity for favourite::Model {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output> {
        let account_url = self
            .find_related(account::Entity)
            .select_only()
            .column(account::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Favourite without associated account");

        let author_account_url = self
            .find_linked(favourite::FavouritedPostAuthor)
            .select_only()
            .column(account::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Post without related account");

        let post_url = self
            .find_related(post::Entity)
            .select_only()
            .column(post::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Favourite without associated post");

        Ok(Activity {
            r#type: ActivityType::Like,
            object: StringOrObject::String(post_url),
            rest: BaseObject {
                context: ap_context(),
                id: self.url,
                attributed_to: Some(StringOrObject::String(account_url)),
                sensitive: false,
                published: self.created_at,
                to: vec![author_account_url, PUBLIC_IDENTIFIER.to_string()],
                cc: vec![],
            },
        })
    }

    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput> {
        let account_url = self
            .find_related(account::Entity)
            .select_only()
            .column(account::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Favourite without associated account");

        let author_account_url = self
            .find_linked(favourite::FavouritedPostAuthor)
            .select_only()
            .column(account::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Post without related account");

        Ok(Activity {
            r#type: ActivityType::Undo,
            rest: BaseObject {
                context: ap_context(),
                id: format!("{}#undo", self.url),
                attributed_to: Some(StringOrObject::String(account_url)),
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
impl IntoActivity for post::Model {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output> {
        let account_url = self
            .find_related(account::Entity)
            .select_only()
            .column(account::Column::Url)
            .into_values::<String, UrlQuery>()
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Post without associated account");

        let created_at = self.created_at;
        let object = self.into_object(state).await?;

        Ok(Activity {
            r#type: ActivityType::Create,
            rest: BaseObject {
                context: ap_context(),
                id: format!("{}/activity", object.id()),
                attributed_to: Some(StringOrObject::String(account_url)),
                sensitive: false,
                published: created_at,
                to: object.to().to_vec(),
                cc: object.cc().to_vec(),
            },
            object: StringOrObject::Object(object),
        })
    }

    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput> {
        let object = self.into_object(state).await?;

        Ok(Activity {
            r#type: ActivityType::Delete,
            rest: BaseObject {
                context: ap_context(),
                id: format!("{}#delete", object.id()),
                sensitive: false,
                attributed_to: None,
                published: Utc::now(),
                to: object.to().to_vec(),
                cc: object.cc().to_vec(),
            },
            object: StringOrObject::String(object.id().to_string()),
        })
    }
}
