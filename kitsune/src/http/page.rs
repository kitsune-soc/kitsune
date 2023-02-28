use crate::error::Result;
use crate::state::Zustand;
use askama::Template;
use futures_util::future::OptionFuture;
use kitsune_db::entity::{
    media_attachments, posts,
    prelude::{Accounts, MediaAttachments},
};
use sea_orm::EntityTrait;
use std::collections::VecDeque;

#[derive(Template)]
#[template(path = "components/post.html", escape = "none")] // Make sure everything is escaped either on submission or in the template
pub struct PostComponent {
    pub display_name: String,
    pub acct: String,
    pub profile_url: String,
    pub profile_picture_url: String,
    pub content: String,
    pub url: String,
    pub attachments: Vec<media_attachments::Model>,
}

impl PostComponent {
    pub async fn prepare(state: &Zustand, post: posts::Model) -> Result<Self> {
        let author = Accounts::find_by_id(post.account_id)
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Post without author");

        let profile_picture_url = OptionFuture::from(
            author
                .avatar_id
                .map(|id| MediaAttachments::find_by_id(id).one(&state.db_conn)),
        )
        .await
        .transpose()?
        .flatten()
        .map(|attachment| attachment.url);

        let mut acct = format!("@{}", author.username);
        if let Some(domain) = author.domain {
            acct.push('@');
            acct.push_str(&domain);
        }

        Ok(Self {
            display_name: author
                .display_name
                .unwrap_or_else(|| author.username.clone()),
            acct,
            profile_url: author.url,
            profile_picture_url: profile_picture_url.unwrap_or_else(|| {
                "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into()
            }),
            content: post.content,
            url: post.url,
            attachments: vec![],
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
