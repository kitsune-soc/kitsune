use super::Account;
use crate::http::graphql::ContextExt;
use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use kitsune_db::model::MediaAttachment as DbMediaAttachment;
use speedy_uuid::Uuid;
use time::OffsetDateTime;

#[derive(Clone, Debug, Eq, PartialEq, SimpleObject)]
#[graphql(complex)]
pub struct MediaAttachment {
    pub id: Uuid,
    #[graphql(skip)]
    pub account_id: Uuid,
    pub content_type: String,
    pub description: Option<String>,
    pub is_sensitive: bool,
    pub created_at: OffsetDateTime,
}

#[ComplexObject]
impl MediaAttachment {
    pub async fn uploader(&self, ctx: &Context<'_>) -> Result<Account> {
        ctx.state()
            .service
            .account
            .get_by_id(self.account_id)
            .await
            .map(Option::unwrap)
            .map(Into::into)
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

impl From<DbMediaAttachment> for MediaAttachment {
    fn from(value: DbMediaAttachment) -> Self {
        Self {
            id: value.id,
            account_id: value.account_id.unwrap(),
            content_type: value.content_type,
            description: value.description,
            is_sensitive: value.is_sensitive,
            created_at: value.created_at.assume_utc(),
        }
    }
}
