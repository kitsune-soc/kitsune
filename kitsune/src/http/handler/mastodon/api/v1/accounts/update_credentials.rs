use crate::{
    error::{ApiError, Result},
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::MastodonMapper,
    service::account::{AccountService, Update},
};
use axum::{
    extract::{Multipart, State},
    Json,
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

    while let Some(field) = multipart.next_field().await? {
        let Some(field_name) = field.name() else {
            continue;
        };

        update = match field_name {
            "display_name" => update.display_name(field.text().await?),
            "note" => update.note(field.text().await?),
            "avatar" => todo!(),
            "header" => todo!(),
            "locked" => update.locked(field.text().await?.parse()?),
            _ => continue,
        };
    }

    let update = update.build().map_err(|_| ApiError::BadRequest)?;
    let account = account_service.update(update).await?;

    Ok(Json(mastodon_mapper.map(account).await?))
}
