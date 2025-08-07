use super::Account;
use crate::http::graphql::ContextExt;
use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{Account as DbAccount, User as DbUser},
    schema::{accounts, users, users_accounts},
    with_connection,
};
use speedy_uuid::Uuid;
use time::OffsetDateTime;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, SimpleObject)]
#[graphql(complex)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[ComplexObject]
impl User {
    pub async fn account(&self, ctx: &Context<'_>) -> Result<Account> {
        let db_pool = &ctx.state().db_pool;
        with_connection!(db_pool, |db_conn| {
            users::table
                .find(self.id)
                .inner_join(users_accounts::table)
                .inner_join(accounts::table.on(accounts::id.eq(users_accounts::account_id)))
                .select(DbAccount::as_select())
                .get_result::<DbAccount>(db_conn)
                .await
                .map(Into::into)
        })
        .map_err(Into::into)
    }
}

impl From<DbUser> for User {
    fn from(value: DbUser) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            created_at: value.created_at.assume_utc(),
            updated_at: value.updated_at.assume_utc(),
        }
    }
}
