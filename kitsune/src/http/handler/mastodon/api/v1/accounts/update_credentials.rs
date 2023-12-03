use crate::{
    error::Result,
    http::{
        extractor::{AuthExtractor, MastodonAuthExtractor},
        util::buffer_multipart_to_tempfile,
    },
};
use axum::{
    extract::{Multipart, State},
    Json,
};
use kitsune_core::error::HttpError;
use kitsune_mastodon::MastodonMapper;
use kitsune_service::{
    account::{AccountService, Update},
    attachment::Upload,
};
use kitsune_type::mastodon::Account;

#[utoipa::path(
    patch,
    path = "/api/v1/accounts/update_credentials",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = 200, description = "Updated account of the user", body = Account),
    )
)]
pub async fn patch(
    State(account_service): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    mut multipart: Multipart,
) -> Result<Json<Account>> {
    let mut update = Update::builder().account_id(user_data.account.id);

    while let Some(mut field) = multipart.next_field().await? {
        let Some(field_name) = field.name() else {
            continue;
        };

        update = match field_name {
            "display_name" => update.display_name(field.text().await?),
            "note" => update.note(field.text().await?),
            "avatar" => {
                let Some(content_type) = field.content_type().map(ToString::to_string) else {
                    return Err(HttpError::BadRequest.into());
                };
                let stream = buffer_multipart_to_tempfile(&mut field).await?;

                let upload = Upload::builder()
                    .account_id(user_data.account.id)
                    .content_type(content_type)
                    .stream(stream)
                    .build()
                    .map_err(|_| HttpError::BadRequest)?;

                update.avatar(upload)
            }
            "header" => {
                let Some(content_type) = field.content_type().map(ToString::to_string) else {
                    return Err(HttpError::BadRequest.into());
                };
                let stream = buffer_multipart_to_tempfile(&mut field).await?;

                let upload = Upload::builder()
                    .account_id(user_data.account.id)
                    .content_type(content_type)
                    .stream(stream)
                    .build()
                    .map_err(|_| HttpError::BadRequest)?;

                update.header(upload)
            }
            "locked" => update.locked(field.text().await?.parse()?),
            _ => continue,
        };
    }

    let update = update.build().map_err(|_| HttpError::BadRequest)?;
    let account = account_service.update(update).await?;

    Ok(Json(mastodon_mapper.map(account).await?))
}
