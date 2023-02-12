use crate::{
    error::{Error, Result},
    state::Zustand,
};
use async_trait::async_trait;
use futures_util::TryStreamExt;
use kitsune_db::entity::{
    accounts, posts, posts_mentions,
    prelude::{Accounts, Favourites, MediaAttachments, Posts, PostsMentions, Reposts},
};
use kitsune_type::mastodon::{account::Source, status::Mention, Account, Status};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter};

#[async_trait]
pub trait IntoMastodon {
    type Output;

    async fn into_mastodon(self, state: &Zustand) -> Result<Self::Output>;
}

#[async_trait]
impl IntoMastodon for accounts::Model {
    type Output = Account;

    async fn into_mastodon(self, state: &Zustand) -> Result<Self::Output> {
        let statuses_count = Posts::find()
            .filter(posts::Column::AccountId.eq(self.id))
            .count(&state.db_conn)
            .await?;
        let mut acct = self.username.clone();
        if let Some(domain) = self.domain {
            acct.push('@');
            acct.push_str(&domain);
        }

        let avatar = if let Some(avatar_id) = self.avatar_id {
            let media_attachment = MediaAttachments::find_by_id(avatar_id)
                .one(&state.db_conn)
                .await?
                .expect("[Bug] User profile picture missing");
            media_attachment.url
        } else {
            "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into()
        };

        let header = if let Some(header_id) = self.header_id {
            let media_attachment = MediaAttachments::find_by_id(header_id)
                .one(&state.db_conn)
                .await?
                .expect("[Bug] User header image missing");
            media_attachment.url
        } else {
            "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into()
        };

        Ok(Account {
            id: self.id,
            acct,
            username: self.username,
            display_name: self.display_name.unwrap_or_default(),
            created_at: self.created_at.into(),
            locked: self.locked,
            note: self.note.unwrap_or_default(),
            url: self.url,
            avatar_static: avatar.clone(),
            avatar,
            header_static: header.clone(),
            header,
            followers_count: 0,
            following_count: 0,
            statuses_count,
            source: Source {
                privacy: "public".into(),
                sensitive: false,
                language: String::new(),
                note: String::new(),
                fields: Vec::new(),
            },
        })
    }
}

#[async_trait]
impl IntoMastodon for posts_mentions::Model {
    type Output = Mention;

    async fn into_mastodon(self, state: &Zustand) -> Result<Self::Output> {
        let account = Accounts::find_by_id(self.account_id)
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Mention without associated account");

        let mut acct = account.username.clone();
        if let Some(ref domain) = account.domain {
            acct.push('@');
            acct.push_str(domain);
        }

        Ok(Mention {
            id: account.id,
            acct,
            username: account.username,
            url: account.url,
        })
    }
}

#[async_trait]
impl IntoMastodon for posts::Model {
    type Output = Status;

    async fn into_mastodon(self, state: &Zustand) -> Result<Self::Output> {
        let account = Accounts::find_by_id(self.account_id)
            .one(&state.db_conn)
            .await?
            .expect("[Bug] Post without associated account")
            .into_mastodon(state)
            .await?;

        let reblog_count = self.find_related(Reposts).count(&state.db_conn).await?;

        let favourites_count = self.find_related(Favourites).count(&state.db_conn).await?;

        let mentions = PostsMentions::find()
            .filter(posts_mentions::Column::PostId.eq(self.id))
            .stream(&state.db_conn)
            .await?
            .map_err(Error::from)
            .and_then(|mention| mention.into_mastodon(state))
            .try_collect()
            .await?;

        Ok(Status {
            id: self.id,
            created_at: self.created_at.into(),
            in_reply_to_account_id: None,
            in_reply_to_id: self.in_reply_to_id,
            sensitive: self.is_sensitive,
            spoiler_text: self.subject,
            visibility: self.visibility.json_repr(),
            uri: self.url.clone(),
            url: self.url,
            replies_count: 0,
            reblog_count,
            favourites_count,
            content: self.content,
            account,
            media_attachments: Vec::new(),
            mentions,
        })
    }
}
