use crate::entity::{
    accounts, accounts_blocks,
    prelude::{Accounts, AccountsBlocks},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, QueryTrait, Select};
use uuid::Uuid;

/// Extension trait for adding permission checks to queries regaring posts
pub trait AccountPermissionCheckExt {
    /// Add checks do omit any posts by users that blocked the user fetching the posts
    #[must_use]
    fn add_blocked_by_checks(self, fetching_account_id: Uuid) -> Self;
}

impl AccountPermissionCheckExt for Select<Accounts> {
    fn add_blocked_by_checks(self, fetching_account_id: Uuid) -> Self {
        self.filter(
            accounts::Column::Id.not_in_subquery(
                AccountsBlocks::find()
                    .filter(accounts_blocks::Column::BlockedAccountId.eq(fetching_account_id))
                    .select_only()
                    .column(accounts_blocks::Column::AccountId)
                    .into_query(),
            ),
        )
    }
}
