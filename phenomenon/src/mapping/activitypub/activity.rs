use super::IntoObject;
use crate::{
    db::model::{account, post},
    error::Result,
    state::Zustand,
};
use async_trait::async_trait;
use chrono::Utc;
use phenomenon_type::ap::{ap_context, helper::StringOrObject, Activity, ActivityType, BaseObject};
use sea_orm::ModelTrait;

#[async_trait]
pub trait IntoActivity {
    type Output;
    type NegateOutput;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output>;
    async fn into_negate_activity(self, state: &Zustand) -> Result<Self::NegateOutput>;
}

#[async_trait]
impl IntoActivity for post::Model {
    type Output = Activity;
    type NegateOutput = Activity;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output> {
        let account = self
            .find_related(account::Entity)
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Post without associated account");
        let object = self.into_object(state).await?;

        Ok(Activity {
            r#type: ActivityType::Create,
            rest: BaseObject {
                context: ap_context(),
                id: format!("{}/activity", object.id()),
                attributed_to: Some(StringOrObject::String(account.url.clone())),
                sensitive: false,
                published: Utc::now(),
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
