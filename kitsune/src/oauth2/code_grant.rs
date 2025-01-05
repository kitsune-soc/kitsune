use komainu::code_grant;
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

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
        todo!();
    }
}
