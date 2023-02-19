use super::MediaAttachment;
use crate::http::graphql::ContextExt;
use async_graphql::{ComplexObject, Context, Error, Result, SimpleObject};
use chrono::{DateTime, Utc};
use kitsune_db::{
    entity::{
        accounts, posts,
        prelude::{Accounts, MediaAttachments, Posts},
    },
    r#trait::PostPermissionCheckExt,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, SimpleObject)]
pub struct Account {
    pub id: Uuid,
    #[graphql(skip)]
    pub avatar_id: Option<Uuid>,
    #[graphql(skip)]
    pub header_id: Option<Uuid>,
    pub display_name: Option<String>,
    pub note: Option<String>,
    pub username: String,
    pub locked: bool,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[ComplexObject]
impl Account {
    pub async fn avatar(&self, ctx: &Context<'_>) -> Result<Option<MediaAttachment>> {
        if let Some(avatar_id) = self.avatar_id {
            MediaAttachments::find_by_id(avatar_id)
                .one(&ctx.state().db_conn)
                .await
                .map(|attachment| attachment.map(Into::into))
                .map_err(Into::into)
        } else {
            Ok(None)
        }
    }

    pub async fn header(&self, ctx: &Context<'_>) -> Result<Option<MediaAttachment>> {
        if let Some(header_id) = self.header_id {
            MediaAttachments::find_by_id(header_id)
                .one(&ctx.state().db_conn)
                .await
                .map(|attachment| attachment.map(Into::into))
                .map_err(Into::into)
        } else {
            Ok(None)
        }
    }

    pub async fn posts(&self, ctx: &Context<'_>) -> Result<Vec<super::Post>> {
        let account = Accounts::find_by_id(self.id)
            .one(&ctx.state().db_conn)
            .await?
            .ok_or_else(|| Error::new("User not present"))?;

        Posts::find()
            .add_permission_checks(None)
            .filter(posts::Column::AccountId.eq(account.id))
            .all(&ctx.state().db_conn)
            .await
            .map(|posts| posts.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }
}

impl From<accounts::Model> for Account {
    fn from(value: accounts::Model) -> Self {
        Self {
            id: value.id,
            avatar_id: value.avatar_id,
            header_id: value.header_id,
            display_name: value.display_name,
            note: value.note,
            username: value.username,
            locked: value.locked,
            url: value.url,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}
