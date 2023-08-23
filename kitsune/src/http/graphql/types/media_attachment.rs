use super::Account;
use crate::http::graphql::ContextExt;
use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{
        account::Account as DbAccount, media_attachment::MediaAttachment as DbMediaAttachment,
    },
    schema::accounts,
};
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
    pub blurhash: Option<String>,
    pub created_at: OffsetDateTime,
}

#[ComplexObject]
impl MediaAttachment {
    pub async fn uploader(&self, ctx: &Context<'_>) -> Result<Account> {
        let mut db_conn = ctx.state().db_pool.get().await?;

        accounts::table
            .find(self.account_id)
            .select(DbAccount::as_select())
            .get_result::<DbAccount>(&mut db_conn)
            .await
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
            account_id: value.account_id,
            content_type: value.content_type,
            description: value.description,
            blurhash: value.blurhash,
            created_at: value.created_at.assume_utc(),
        }
    }
}
