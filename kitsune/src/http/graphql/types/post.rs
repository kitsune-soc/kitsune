use super::{Account, MediaAttachment, Visibility};
use crate::http::graphql::ContextExt;
use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::TryStreamExt;
use kitsune_db::{
    model::{MediaAttachment as DbMediaAttachment, Post as DbPost},
    schema::{media_attachments, posts_media_attachments},
    with_connection,
};
use speedy_uuid::Uuid;
use time::OffsetDateTime;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, SimpleObject)]
#[graphql(complex)]
pub struct Post {
    pub id: Uuid,
    #[graphql(skip)]
    pub account_id: Uuid,
    #[graphql(skip)]
    pub in_reply_to_id: Option<Uuid>,
    pub subject: Option<String>,
    pub content: String,
    pub visibility: Visibility,
    pub url: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[ComplexObject]
impl Post {
    pub async fn account(&self, ctx: &Context<'_>) -> Result<Account> {
        ctx.state()
            .service
            .account
            .get_by_id(self.account_id)
            .await
            .map(Option::unwrap)
            .map(Into::into)
            .map_err(Into::into)
    }

    pub async fn attachments(&self, ctx: &Context<'_>) -> Result<Vec<MediaAttachment>> {
        let db_pool = &ctx.state().db_pool;
        let attachments = with_connection!(db_pool, |db_conn| {
            media_attachments::table
                .inner_join(posts_media_attachments::table)
                .filter(posts_media_attachments::post_id.eq(self.id))
                .select(DbMediaAttachment::as_select())
                .load_stream(db_conn)
                .await?
                .map_ok(Into::into)
                .try_collect()
                .await
        })?;

        Ok(attachments)
    }
}

impl From<DbPost> for Post {
    fn from(value: DbPost) -> Self {
        Self {
            id: value.id,
            account_id: value.account_id,
            in_reply_to_id: value.in_reply_to_id,
            subject: value.subject,
            content: value.content,
            visibility: value.visibility.into(),
            url: value.url,
            created_at: value.created_at.assume_utc(),
            updated_at: value.updated_at.assume_utc(),
        }
    }
}
