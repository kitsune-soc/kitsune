use super::{MediaAttachment, Post};
use crate::{http::graphql::ContextExt, service::account::GetPosts};
use async_graphql::{
    connection::{self, Connection, Edge},
    ComplexObject, Context, Result, SimpleObject,
};
use diesel::{OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use futures_util::TryStreamExt;
use kitsune_db::{
    model::{
        account::Account as DbAccount, media_attachment::MediaAttachment as DbMediaAttachment,
    },
    schema::media_attachments,
};
use speedy_uuid::Uuid;
use time::OffsetDateTime;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, SimpleObject)]
#[graphql(complex)]
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

    pub async fn posts(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<Uuid, Post>> {
        connection::query(
            after,
            before,
            first,
            last,
            |after, before, first, _last| async move {
                let account_service = &ctx.state().service.account;
                let get_posts = GetPosts::builder()
                    .account_id(self.id)
                    .fetching_account_id(ctx.user_data().ok().map(|user_data| user_data.account.id))
                    .max_id(after)
                    .min_id(before);

                let get_posts = if let Some(first) = first {
                    get_posts.limit(first).build()
                } else {
                    get_posts.build()
                };

                let mut post_stream = account_service
                    .get_posts(get_posts)
                    .await?
                    .map_ok(Post::from);

                let mut connection = Connection::new(true, true); // TODO: Set actual values
                while let Some(post) = post_stream.try_next().await? {
                    connection.edges.push(Edge::new(post.id, post));
                }

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
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
