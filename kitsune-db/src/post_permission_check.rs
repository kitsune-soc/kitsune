use derive_builder::Builder;
use uuid::Uuid;

/// Parameters for adding a permission check to a post select query
#[derive(Builder, Clone)]
pub struct PermissionCheck {
    /// ID of the account that is fetching the posts
    #[builder(default)]
    #[doc(hidden)]
    pub fetching_account_id: Option<Uuid>,

    /// Include unlisted posts in the results
    ///
    /// Default: true
    #[builder(default = "true")]
    #[doc(hidden)]
    pub include_unlisted: bool,
}

impl PermissionCheck {
    /// Create a new permission check builder
    #[must_use]
    pub fn builder() -> PermissionCheckBuilder {
        PermissionCheckBuilder::default()
    }
}

impl Default for PermissionCheck {
    fn default() -> Self {
        Self::builder().build().unwrap()
    }
}

#[macro_export]
macro_rules! add_post_permission_check {
    ($permission_opts:expr => $query:expr) => {{
        use diesel::{
            sql_types::Bool, BoolExpressionMethods, BoxableExpression, ExpressionMethods, QueryDsl,
        };
        use $crate::{
            model::post::Visibility,
            schema::{accounts_follows, posts, posts_mentions},
        };

        let mut permission_opts = &$permission_opts;
        let mut post_condition: Box<dyn BoxableExpression<_, _, SqlType = Bool>> =
            Box::new(posts::visibility.eq(Visibility::Public));

        if permission_opts.include_unlisted {
            post_condition =
                Box::new(post_condition.or(posts::visibility.eq(Visibility::Unlisted)));
        }

        if let Some(fetching_account_id) = permission_opts.fetching_account_id {
            post_condition = Box::new(
                post_condition.or(
                    // The post is owned by the user
                    (posts::account_id.eq(fetching_account_id))
                        .or(
                            // Post is follower-only, and the user is following the author
                            (posts::visibility.eq(Visibility::FollowerOnly).and(
                                posts::account_id.eq_any(
                                    accounts_follows::table
                                        .filter(
                                            accounts_follows::follower_id
                                                .eq(fetching_account_id)
                                                .and(accounts_follows::approved_at.is_not_null()),
                                        )
                                        .select(accounts_follows::account_id),
                                ),
                            )),
                        )
                        .or(
                            // Post is mention-only, and user is mentioned in the post
                            (posts::visibility.eq(Visibility::MentionOnly).and(
                                posts::id.eq_any(
                                    posts_mentions::table
                                        .filter(posts_mentions::account_id.eq(fetching_account_id))
                                        .select(posts_mentions::post_id),
                                ),
                            )),
                        ),
                ),
            );
        }

        $query.filter(post_condition)
    }};
}
