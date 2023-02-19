use crate::error::{Error, Result};
use derive_builder::Builder;
use futures_util::{Stream, TryStreamExt};
use kitsune_db::{
    entity::{posts, prelude::Posts},
    r#trait::PostPermissionCheckExt,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

#[derive(Builder, Clone)]
pub struct GetPublic {
    #[builder(default, setter(strip_option))]
    fetching_account_id: Option<Uuid>,

    #[builder(default, setter(strip_option))]
    max_id: Option<Uuid>,

    #[builder(default, setter(strip_option))]
    min_id: Option<Uuid>,

    #[builder(default)]
    only_local: bool,

    #[builder(default)]
    only_remote: bool,
}

impl GetPublic {
    #[must_use]
    pub fn builder() -> GetPublicBuilder {
        GetPublicBuilder::default()
    }
}

#[derive(Builder, Clone)]
pub struct TimelineService {
    db_conn: DatabaseConnection,
}

impl TimelineService {
    #[must_use]
    pub fn builder() -> TimelineServiceBuilder {
        TimelineServiceBuilder::default()
    }

    pub async fn get_public(
        &self,
        get_public: GetPublic,
    ) -> Result<impl Stream<Item = Result<posts::Model>> + '_> {
        let mut query = Posts::find().add_permission_checks(get_public.fetching_account_id);

        if let Some(max_id) = get_public.max_id {
            query = query.filter(posts::Column::Id.lt(max_id));
        }
        if let Some(min_id) = get_public.min_id {
            query = query.filter(posts::Column::Id.gt(min_id));
        }

        if get_public.only_local {
            query = query.filter(posts::Column::IsLocal.eq(true));
        } else if get_public.only_remote {
            query = query.filter(posts::Column::IsLocal.eq(false));
        }

        Ok(query.stream(&self.db_conn).await?.map_err(Error::from))
    }
}
