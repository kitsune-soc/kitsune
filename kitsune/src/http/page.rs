use crate::{
    error::{Error, Result},
    state::Zustand,
};
use askama::Template;
use diesel::{BelongingToDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{future::OptionFuture, TryStreamExt};
use kitsune_common::try_join;
use kitsune_db::{
    model::{
        account::Account,
        media_attachment::{MediaAttachment as DbMediaAttachment, PostMediaAttachment},
        post::Post,
    },
    schema::{accounts, media_attachments},
};
use std::collections::VecDeque;

pub struct MediaAttachment {
    pub content_type: String,
    pub description: Option<String>,
    pub url: String,
}

#[derive(Template)]
#[template(path = "components/post.html", escape = "none")] // Make sure everything is escaped either on submission or in the template
pub struct PostComponent {
    pub display_name: String,
    pub acct: String,
    pub profile_url: String,
    pub profile_picture_url: String,
    pub content: String,
    pub url: String,
    pub attachments: Vec<MediaAttachment>,
}

impl PostComponent {
    pub async fn prepare(state: &Zustand, post: Post) -> Result<Self> {
        let mut db_conn = state.db_conn.get().await?;

        let author_fut = accounts::table
            .find(post.account_id)
            .select(Account::as_select())
            .get_result::<Account>(&mut db_conn);

        let attachments_stream_fut = PostMediaAttachment::belonging_to(&post)
            .inner_join(media_attachments::table)
            .select(DbMediaAttachment::as_select())
            .load_stream::<DbMediaAttachment>(&mut db_conn);

        let (author, attachments_stream) = try_join!(author_fut, attachments_stream_fut)?;

        let attachments = attachments_stream
            .map_err(Error::from)
            .and_then(|attachment| async move {
                let url = state.service.attachment.get_url(attachment.id).await?;

                Ok(MediaAttachment {
                    content_type: attachment.content_type,
                    description: attachment.description,
                    url,
                })
            })
            .try_collect()
            .await?;

        let profile_picture_url = OptionFuture::from(
            author
                .avatar_id
                .map(|id| state.service.attachment.get_url(id)),
        )
        .await
        .transpose()?;

        let mut acct = format!("@{}", author.username);
        if !author.local {
            acct.push('@');
            acct.push_str(&author.domain);
        }

        Ok(Self {
            attachments,
            display_name: author
                .display_name
                .unwrap_or_else(|| author.username.clone()),
            acct,
            profile_url: author.url,
            profile_picture_url: profile_picture_url
                .unwrap_or_else(|| state.service.url.default_avatar_url()),
            content: post.content,
            url: post.url,
        })
    }
}

#[derive(Template)]
#[template(path = "pages/posts.html", escape = "none")]
pub struct PostPage {
    pub ancestors: VecDeque<PostComponent>,
    pub post: PostComponent,
    pub descendants: Vec<PostComponent>,
    pub version: &'static str,
}

#[derive(Template)]
#[template(path = "pages/users.html", escape = "none")]
pub struct UserPage {
    pub acct: String,
    pub display_name: String,
    pub profile_picture_url: String,
    pub bio: String,
    pub posts: Vec<PostComponent>,
}
