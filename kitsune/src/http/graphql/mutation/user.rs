use super::handle_upload;
use crate::{
    http::graphql::{types::Account, ContextExt},
    sanitize::CleanHtmlExt,
};
use async_graphql::{Context, Error, Object, Result, Upload};
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use kitsune_db::schema::accounts;

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
        let mut db_conn = ctx.state().db_conn.get().await?;
        let user_data = ctx.user_data()?;
        let mut update_query = diesel::update(&user_data.account);

        if let Some(mut display_name) = display_name {
            update_query = if display_name.is_empty() {
                update_query.set(accounts::display_name.eq(None))
            } else {
                display_name.clean_html();
                update_query.set(accounts::display_name.eq(display_name))
            };
        }

        if let Some(mut note) = note {
            update_query = if note.is_empty() {
                update_query.set(accounts::note.eq(None))
            } else {
                note.clean_html();
                update_query.set(accounts::note.eq(note))
            };
        }

        if let Some(avatar) = avatar {
            let media_attachment = handle_upload(ctx, avatar, None).await?;
            update_query = update_query.set(accounts::avatar_id.eq(media_attachment.id));
        }

        if let Some(header) = header {
            let media_attachment = handle_upload(ctx, header, None).await?;
            update_query = update_query.set(accounts::header_id.eq(media_attachment.id));
        }

        if let Some(locked) = locked {
            update_query = update_query.set(accounts::locked.eq(locked));
        }

        update_query
            .execute(&mut db_conn)
            .await
            .map(Into::into)
            .map_err(Error::from)
    }
}
