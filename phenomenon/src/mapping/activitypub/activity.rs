use crate::{
    db::model::{account, post},
    error::Result,
    state::Zustand,
};
use async_trait::async_trait;
use chrono::Utc;
use phenomenon_type::ap::{helper::StringOrObject, Activity, ActivityType, BaseObject};
use sea_orm::ModelTrait;

use super::IntoObject;

#[async_trait]
pub trait IntoActivity {
    type Output;

    async fn into_activity(self, state: &Zustand) -> Result<Self::Output>;
}

#[async_trait]
impl IntoActivity for post::Model {
    type Output = Activity;

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
                id: format!("{}/activity", object.id()),
                attributed_to: Some(StringOrObject::String(account.url.clone())),
                published: Utc::now(),
                to: object.to().to_vec(),
                cc: object.cc().to_vec(),
                ..Default::default()
            },
            object: StringOrObject::Object(object),
        })
    }
}
