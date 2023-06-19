use super::Visibility;
use crate::http::graphql::ContextExt;
use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{account::Account as DbAccount, post::Post as DbPost},
    schema::accounts,
};
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
        let mut db_conn = ctx.state().db_conn.get().await?;

        Ok(accounts::table
            .find(self.account_id)
            .select(DbAccount::as_select())
            .get_result::<DbAccount>(&mut db_conn)
            .await?
            .into())
    }
}

impl From<DbPost> for Post {
    fn from(value: DbPost) -> Self {
        Self {
            id: value.id,
            account_id: value.account_id,
            in_reply_to_id: value.in_reply_to_id,
            is_sensitive: value.is_sensitive,
            subject: value.subject,
            content: value.content,
            visibility: value.visibility.into(),
            url: value.url,
            created_at: value.created_at.assume_utc(),
            updated_at: value.updated_at.assume_utc(),
        }
    }
}
