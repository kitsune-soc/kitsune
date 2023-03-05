use super::Account;
use crate::http::graphql::ContextExt;
use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use chrono::{DateTime, Utc};
use kitsune_db::entity::{media_attachments, prelude::Accounts};
use sea_orm::EntityTrait;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, SimpleObject)]
#[graphql(complex)]
pub struct MediaAttachment {
    pub id: Uuid,
    #[graphql(skip)]
    pub account_id: Uuid,
    pub content_type: String,
    pub description: Option<String>,
    pub blurhash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[ComplexObject]
impl MediaAttachment {
    pub async fn uploader(&self, ctx: &Context<'_>) -> Result<Option<Account>> {
        Accounts::find_by_id(self.account_id)
            .one(&ctx.state().db_conn)
            .await
            .map(|account| account.map(Into::into))
            .map_err(Into::into)
    }

    pub async fn url(&self, ctx: &Context<'_>) -> Result<String> {
        ctx.state()
            .service
            .attachment
            .get_url(self.id)
            .await
            .map_err(Into::into)
    }
}

impl From<media_attachments::Model> for MediaAttachment {
    fn from(value: media_attachments::Model) -> Self {
        Self {
            id: value.id,
            account_id: value.account_id,
            content_type: value.content_type,
            description: value.description,
            blurhash: value.blurhash,
            created_at: value.created_at.into(),
        }
    }
}
