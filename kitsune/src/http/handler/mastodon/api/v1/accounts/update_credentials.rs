use crate::http::{
    extractor::{AuthExtractor, MastodonAuthExtractor},
    util::buffer_multipart_to_tempfile,
};
use axum::{
    Json,
    extract::{Multipart, State},
};
use kitsune_error::{ErrorType, Result, bail, kitsune_error};
use kitsune_mastodon::MastodonMapper;
use kitsune_service::{
    account::{AccountService, Update},
    attachment::Upload,
};
use kitsune_type::mastodon::Account;

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
                    bail!(type = ErrorType::BadRequest, "invalid content-type");
                };
                let stream = buffer_multipart_to_tempfile(&mut field).await?;

                let upload = Upload::builder()
                    .account_id(user_data.account.id)
                    .content_type(content_type)
                    .stream(stream)
                    .build()
                    .unwrap();

                update.avatar(upload)
            }
            "header" => {
                let Some(content_type) = field.content_type().map(ToString::to_string) else {
                    bail!(type = ErrorType::BadRequest, "invalid content-type");
                };
                let stream = buffer_multipart_to_tempfile(&mut field).await?;

                let upload = Upload::builder()
                    .account_id(user_data.account.id)
                    .content_type(content_type)
                    .stream(stream)
                    .build()
                    .unwrap();

                update.header(upload)
            }
            "locked" => update.locked(field.text().await?.parse()?),
            _ => continue,
        };
    }

    let update = update.build().map_err(|err| {
        kitsune_error!(
            type = ErrorType::BadRequest.with_body(err.to_string()),
            "missing upload field"
        )
    })?;
    let account = account_service.update(update).await?;

    Ok(Json(mastodon_mapper.map(account).await?))
}
