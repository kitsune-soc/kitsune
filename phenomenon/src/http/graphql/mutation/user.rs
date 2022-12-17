use super::handle_upload;
use crate::{db::model::account, http::graphql::ContextExt, sanitize::CleanHtmlExt};
use async_graphql::{Context, Error, Object, Result, Upload};
use sea_orm::{ActiveModelTrait, ActiveValue};

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
    ) -> Result<account::Model> {
        let state = ctx.state();
        let user = ctx.user_data()?;
        let mut active_user = account::ActiveModel {
            id: ActiveValue::Set(user.account.id),
            ..Default::default()
        };

        if let Some(mut display_name) = display_name {
            active_user.display_name = if display_name.is_empty() {
                ActiveValue::Set(None)
            } else {
                display_name.clean_html();
                ActiveValue::Set(Some(display_name))
            };
        }

        if let Some(mut note) = note {
            active_user.note = if note.is_empty() {
                ActiveValue::Set(None)
            } else {
                note.clean_html();
                ActiveValue::Set(Some(note))
            };
        }

        if let Some(avatar) = avatar {
            let media_attachment = handle_upload(ctx, avatar).await?;
            active_user.avatar_id = ActiveValue::Set(Some(media_attachment.id));
        }

        if let Some(header) = header {
            let media_attachment = handle_upload(ctx, header).await?;
            active_user.header_id = ActiveValue::Set(Some(media_attachment.id));
        }

        active_user
            .update(&state.db_conn)
            .await
            .map_err(Error::from)
    }
}
