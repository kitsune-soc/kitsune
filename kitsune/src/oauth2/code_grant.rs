use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use iso8601_timestamp::Timestamp;
use kitsune_db::{model::oauth2, schema::oauth2_authorization_codes, with_connection};
use kitsune_util::generate_secret;
use komainu::code_grant;
use speedy_uuid::Uuid;
use std::{str::FromStr, time::Duration};
use trials::attempt;
use typed_builder::TypedBuilder;

const CODE_TTL: Duration = Duration::from_secs(10 * 60);

#[derive(TypedBuilder)]
pub struct Issuer {
    db_pool: kitsune_db::PgPool,
}

impl code_grant::Issuer for Issuer {
    type UserId = Uuid;

    async fn issue_code(
        &self,
        user_id: Self::UserId,
        pre_authorization: komainu::AuthInstruction<'_, '_>,
    ) -> Result<String, code_grant::GrantError> {
        let client_id = Uuid::from_str(&pre_authorization.client.client_id).unwrap();
        let scopes = pre_authorization.scopes.to_string();

        let result: Result<_, kitsune_error::Error> = attempt! { async
            with_connection!(self.db_pool, |db_conn| {
                diesel::insert_into(oauth2_authorization_codes::table)
                    .values(oauth2::NewAuthorizationCode {
                        code: generate_secret().as_str(),
                        user_id,
                        application_id: client_id,
                        scopes: &scopes,
                        expires_at: Timestamp::now_utc() + CODE_TTL,
                    })
                    .returning(oauth2::AuthorizationCode::as_returning())
                    .get_result::<oauth2::AuthorizationCode>(db_conn)
                    .await
            })?
        };

        // ToDo: error handling
        let token = result.unwrap();

        Ok(token.code)
    }
}
