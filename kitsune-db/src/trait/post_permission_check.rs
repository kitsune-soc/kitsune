use crate::{
    custom::Visibility,
    entity::{
        accounts, accounts_followers, posts, posts_mentions,
        prelude::{AccountsFollowers, Posts},
    },
};
use sea_orm::{
    sea_query::{Expr, IntoCondition, JoinType},
    ColumnTrait, EntityTrait, QueryFilter, QuerySelect, QueryTrait, RelationTrait, Select,
};
use uuid::Uuid;

/// Extension trait for adding permission checks to queries regaring posts
pub trait PostPermissionCheckExt {
    /// Add permission checks to the query
    #[must_use]
    fn add_permission_checks(self, fetching_account_id: Option<Uuid>) -> Self;
}

impl PostPermissionCheckExt for Select<Posts> {
    fn add_permission_checks(mut self, fetching_account_id: Option<Uuid>) -> Self {
        let mut post_filter = posts::Column::Visibility
            .eq(Visibility::Public)
            .or(posts::Column::Visibility.eq(Visibility::Unlisted));

        if let Some(fetching_account_id) = fetching_account_id {
            // The post is owned by the user
            post_filter = post_filter.or(posts::Column::AccountId.eq(fetching_account_id));

            // Post is follower-only, and the user is following the author
            self = self.join(
                JoinType::LeftJoin,
                posts::Relation::Accounts
                    .def()
                    .on_condition(move |posts_left, accounts_right| {
                        Expr::col((posts_left, posts::Column::Visibility))
                            .eq(Visibility::FollowerOnly)
                            .and(
                                Expr::col((accounts_right, accounts::Column::Id)).in_subquery(
                                    AccountsFollowers::find()
                                        .filter(
                                            accounts_followers::Column::FollowerId
                                                .eq(fetching_account_id),
                                        )
                                        .filter(
                                            accounts_followers::Column::ApprovedAt.is_not_null(),
                                        )
                                        .select_only()
                                        .column(accounts_followers::Column::AccountId)
                                        .into_query(),
                                ),
                            )
                            .into_condition()
                    }),
            );

            // Post is mention-only, and user is mentioned in the post
            self = self.join(
                JoinType::LeftJoin,
                posts_mentions::Relation::Posts.def().rev().on_condition(
                    move |posts_left, mentions_right| {
                        Expr::col((posts_left, posts::Column::Visibility))
                            .eq(Visibility::MentionOnly)
                            .and(
                                Expr::col((mentions_right, posts_mentions::Column::AccountId))
                                    .eq(fetching_account_id),
                            )
                            .into_condition()
                    },
                ),
            );
        }

        self.filter(post_filter)
    }
}
