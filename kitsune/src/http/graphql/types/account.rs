use super::MediaAttachment;
use crate::{http::graphql::ContextExt, service::account::GetPosts};
use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use diesel::{OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use futures_util::TryStreamExt;
use kitsune_db::{
    model::{
        account::Account as DbAccount, media_attachment::MediaAttachment as DbMediaAttachment,
    },
    schema::media_attachments,
};
use time::OffsetDateTime;
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
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[ComplexObject]
impl Account {
    pub async fn avatar(&self, ctx: &Context<'_>) -> Result<Option<MediaAttachment>> {
        let mut db_conn = ctx.state().db_conn.get().await?;

        if let Some(avatar_id) = self.avatar_id {
            media_attachments::table
                .find(avatar_id)
                .get_result::<DbMediaAttachment>(&mut db_conn)
                .await
                .optional()
                .map(|attachment| attachment.map(Into::into))
                .map_err(Into::into)
        } else {
            Ok(None)
        }
    }

    pub async fn header(&self, ctx: &Context<'_>) -> Result<Option<MediaAttachment>> {
        let mut db_conn = ctx.state().db_conn.get().await?;

        if let Some(header_id) = self.header_id {
            media_attachments::table
                .find(header_id)
                .get_result::<DbMediaAttachment>(&mut db_conn)
                .await
                .optional()
                .map(|attachment| attachment.map(Into::into))
                .map_err(Into::into)
        } else {
            Ok(None)
        }
    }

    pub async fn posts(&self, ctx: &Context<'_>) -> Result<Vec<super::Post>> {
        let account_service = &ctx.state().service.account;
        let get_posts = GetPosts::builder().account_id(self.id).limit(40).build();
        let posts = account_service
            .get_posts(get_posts)
            .await?
            .map_ok(Into::into)
            .try_collect()
            .await?;

        Ok(posts)
    }
}

impl From<DbAccount> for Account {
    fn from(value: DbAccount) -> Self {
        Self {
            id: value.id,
            avatar_id: value.avatar_id,
            header_id: value.header_id,
            display_name: value.display_name,
            note: value.note,
            username: value.username,
            locked: value.locked,
            url: value.url,
            created_at: value.created_at.assume_utc(),
            updated_at: value.updated_at.assume_utc(),
        }
    }
}
