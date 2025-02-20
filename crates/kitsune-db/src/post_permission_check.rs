use crate::{
    model::post::Visibility,
    schema::{accounts_follows, posts, posts_mentions},
};
use diesel::{
    BoolExpressionMethods, BoxableExpression, ExpressionMethods,
    pg::Pg,
    query_dsl::{filter_dsl::FilterDsl, select_dsl::SelectDsl},
    sql_types::Bool,
};
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

/// Parameters for adding a permission check to a post select query
#[derive(Clone, Copy, TypedBuilder)]
pub struct PermissionCheck {
    /// ID of the account that is fetching the posts
    #[builder(default)]
    fetching_account_id: Option<Uuid>,

    /// Include unlisted posts in the results
    ///
    /// Default: true
    #[builder(default = true)]
    include_unlisted: bool,
}

impl Default for PermissionCheck {
    fn default() -> Self {
        Self::builder().build()
    }
}

pub trait PostPermissionCheckExt {
    type Output;

    fn add_post_permission_check(self, permission_check: PermissionCheck) -> Self::Output;
}

impl<T> PostPermissionCheckExt for T
where
    T: FilterDsl<Box<dyn BoxableExpression<posts::table, Pg, SqlType = Bool>>>,
{
    type Output = T::Output;

    fn add_post_permission_check(self, permission_check: PermissionCheck) -> Self::Output {
        let mut post_condition: Box<dyn BoxableExpression<_, _, SqlType = _>> =
            Box::new(posts::visibility.eq(Visibility::Public));

        if permission_check.include_unlisted {
            post_condition =
                Box::new(post_condition.or(posts::visibility.eq(Visibility::Unlisted)));
        }

        if let Some(fetching_account_id) = permission_check.fetching_account_id {
            post_condition = Box::new(
                post_condition.or(
                    // The post is owned by the user
                    (posts::account_id.eq(fetching_account_id))
                        .or(
                            // Post is follower-only, and the user is following the author
                            posts::visibility.eq(Visibility::FollowerOnly).and(
                                posts::account_id.eq_any(
                                    accounts_follows::table
                                        .filter(
                                            accounts_follows::follower_id
                                                .eq(fetching_account_id)
                                                .and(accounts_follows::approved_at.is_not_null()),
                                        )
                                        .select(accounts_follows::account_id),
                                ),
                            ),
                        )
                        .or(
                            // Post is mention-only, and user is mentioned in the post
                            posts::visibility.eq(Visibility::MentionOnly).and(
                                posts::id.eq_any(
                                    posts_mentions::table
                                        .filter(posts_mentions::account_id.eq(fetching_account_id))
                                        .select(posts_mentions::post_id),
                                ),
                            ),
                        ),
                ),
            );
        }

        self.filter(post_condition)
    }
}
