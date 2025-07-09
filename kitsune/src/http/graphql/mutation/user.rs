use super::handle_upload;
use crate::http::graphql::{ContextExt, types::Account};
use async_graphql::{Context, Error, Object, Result, Upload};
use kitsune_service::account::Update;

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserMutation {
    pub async fn update_user(
        &self,
        ctx: &Context<'_>,
        display_name: Option<String>,
        note: Option<String>,
        avatar: Option<Upload>,
        header: Option<Upload>,
        locked: Option<bool>,
    ) -> Result<Account> {
        let account_service = &ctx.state().service.account;
        let mut account_update = Update::builder();

        if let Some(display_name) = display_name {
            account_update = account_update.display_name(display_name);
        }

        if let Some(note) = note {
            account_update = account_update.note(note);
        }

        if let Some(avatar) = avatar {
            let media_attachment = handle_upload(ctx, avatar, None)?;
            account_update = account_update.avatar(media_attachment);
        }

        if let Some(header) = header {
            let media_attachment = handle_upload(ctx, header, None)?;
            account_update = account_update.header(media_attachment);
        }

        if let Some(locked) = locked {
            account_update = account_update.locked(locked);
        }

        account_service
            .update(account_update.build()?)
            .await
            .map(Account::from)
            .map_err(Error::from)
    }
}
