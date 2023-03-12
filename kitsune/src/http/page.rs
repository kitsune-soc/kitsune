use crate::error::{Error, Result};
use crate::state::Zustand;
use askama::Template;
use futures_util::{future::OptionFuture, TryStreamExt};
use kitsune_db::entity::{
    posts,
    prelude::{Accounts, MediaAttachments},
};
use sea_orm::{EntityTrait, ModelTrait};
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
    pub async fn prepare(state: &Zustand, post: posts::Model) -> Result<Self> {
        let author = Accounts::find_by_id(post.account_id)
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Post without author");

        let attachments = post
            .find_related(MediaAttachments)
            .stream(&state.db_conn)
            .await?
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
        if let Some(domain) = author.domain {
            acct.push('@');
            acct.push_str(&domain);
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
