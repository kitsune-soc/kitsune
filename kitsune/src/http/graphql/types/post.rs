use super::Visibility;
use crate::http::graphql::ContextExt;
use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, SimpleObject)]
pub struct Post {
    pub id: Uuid,
    #[graphql(skip)]
    pub account_id: Uuid,
    #[graphql(skip)]
    pub in_reply_to_id: Option<Uuid>,
    pub is_sensitive: bool,
    pub subject: Option<String>,
    pub content: String,
    pub visibility: Visibility,
    pub url: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[ComplexObject]
impl Post {
    pub async fn account(&self, ctx: &Context<'_>) -> Result<super::Account> {
        Ok(Accounts::find_by_id(self.account_id)
            .one(&ctx.state().db_conn)
            .await?
            .expect("[Bug] Post without associated user encountered")
            .into())
    }
}

impl From<posts::Model> for Post {
    fn from(value: posts::Model) -> Self {
        Self {
            id: value.id,
            account_id: value.account_id,
            in_reply_to_id: value.in_reply_to_id,
            is_sensitive: value.is_sensitive,
            subject: value.subject,
            content: value.content,
            visibility: value.visibility.into(),
            url: value.url,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
