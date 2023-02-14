use super::search::SearchService;
use crate::{
    cache::Cache,
    config::Configuration,
    error::{ApiError, Error, Result},
    job::{
        deliver::{
            create::CreateDeliveryContext, delete::DeleteDeliveryContext,
            favourite::FavouriteDeliveryContext,
        },
        Job,
    },
    resolve::PostResolver,
    sanitize::CleanHtmlExt,
};
use chrono::Utc;
use derive_builder::Builder;
use futures_util::FutureExt;
use kitsune_db::{
    custom::{JobState, Role, Visibility},
    entity::{
        accounts, accounts_followers, favourites, jobs, posts, posts_mentions,
        prelude::{AccountsFollowers, Posts, UsersRoles},
        users_roles,
    },
};
use pulldown_cmark::{html, Options, Parser};
use sea_orm::{
    sea_query::{Expr, IntoCondition},
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, JoinType,
    PaginatorTrait, QueryFilter, QuerySelect, QueryTrait, RelationTrait, Select, TransactionTrait,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Builder)]
pub struct CreatePost {
    /// ID of the author
    ///
    /// This is not validated. Make sure this is a valid and verified value.
    author_id: Uuid,

    /// ID of the post this post is replying to
    ///
    /// This is validated. If you pass in an non-existent ID, it will be ignored.
    #[builder(default, setter(strip_option))]
    in_reply_to_id: Option<Uuid>,

    /// Mark this post as sensitive
    ///
    /// Defaults to false
    #[builder(default)]
    sensitive: bool,

    #[builder(default, setter(strip_option))]
    /// Subject of the post
    ///
    /// This is optional
    subject: Option<String>,

    /// Content of the post
    content: String,

    #[builder(default = "Visibility::Public")]
    /// Visibility of the post
    ///
    /// Defaults to public
    visibility: Visibility,
}

impl CreatePost {
    #[must_use]
    pub fn builder() -> CreatePostBuilder {
        CreatePostBuilder::default()
    }
}

#[derive(Clone, Builder)]
pub struct DeletePost {
    /// ID of the account that is associated with the user
    account_id: Uuid,

    /// ID of the user that requests the deletion
    ///
    /// Defaults to none
    #[builder(default, setter(strip_option))]
    user_id: Option<Uuid>,

    /// ID of the post that is supposed to be deleted
    post_id: Uuid,
}

impl DeletePost {
    #[must_use]
    pub fn builder() -> DeletePostBuilder {
        DeletePostBuilder::default()
    }
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct PostService<
    S = Arc<dyn SearchService + Send + Sync>,
    FPC = Arc<dyn Cache<str, posts::Model> + Send + Sync>,
    FUC = Arc<dyn Cache<str, accounts::Model> + Send + Sync>,
    WC = Arc<dyn Cache<str, String> + Send + Sync>,
> {
    config: Configuration,
    db_conn: DatabaseConnection,
    search_service: S,
    post_resolver: PostResolver<S, FPC, FUC, WC>,
}

impl<S, FPC, FUC, WC> PostService<S, FPC, FUC, WC>
where
    S: SearchService,
    FPC: Cache<str, posts::Model>,
    FUC: Cache<str, accounts::Model>,
    WC: Cache<str, String>,
{
    #[must_use]
    pub fn builder() -> PostServiceBuilder<S, FPC, FUC, WC> {
        PostServiceBuilder::default()
    }

    /// Create a new post and deliver it to the followers
    ///
    /// # Panics
    ///
    /// This should never ever panic. If it does, create a bug report.
    pub async fn create(&self, create_post: CreatePost) -> Result<posts::Model> {
        let content = {
            let parser = Parser::new_ext(&create_post.content, Options::all());
            let mut buf = String::new();

            html::push_html(&mut buf, parser);
            buf.clean_html();

            buf
        };

        let (mentioned_account_ids, content) = self.post_resolver.resolve(&content).await?;

        let id = Uuid::now_v7();
        let url = format!("https://{}/posts/{id}", self.config.domain);

        let post = self
            .db_conn
            .transaction(move |tx| {
                async move {
                    let in_reply_to_id = if let Some(in_reply_to_id) = create_post.in_reply_to_id {
                        (Posts::find_by_id(in_reply_to_id).count(tx).await? != 0)
                            .then_some(in_reply_to_id)
                    } else {
                        None
                    };

                    let post = posts::Model {
                        id,
                        account_id: create_post.author_id,
                        in_reply_to_id,
                        subject: create_post.subject,
                        content,
                        is_sensitive: create_post.sensitive,
                        visibility: create_post.visibility,
                        is_local: true,
                        url,
                        created_at: Utc::now().into(),
                        updated_at: Utc::now().into(),
                    }
                    .into_active_model()
                    .insert(tx)
                    .await?;

                    for account_id in mentioned_account_ids {
                        posts_mentions::Model {
                            account_id,
                            post_id: post.id,
                        }
                        .into_active_model()
                        .insert(tx)
                        .await?;
                    }

                    let job_context =
                        Job::DeliverCreate(CreateDeliveryContext { post_id: post.id });

                    jobs::Model {
                        id: Uuid::now_v7(),
                        state: JobState::Queued,
                        run_at: Utc::now().into(),
                        context: serde_json::to_value(job_context).unwrap(),
                        fail_count: 0,
                        created_at: Utc::now().into(),
                        updated_at: Utc::now().into(),
                    }
                    .into_active_model()
                    .insert(tx)
                    .await?;

                    Ok(post)
                }
                .boxed()
            })
            .await?;

        if create_post.visibility == Visibility::Public
            || create_post.visibility == Visibility::Unlisted
        {
            self.search_service
                .add_to_index(post.clone().into())
                .await?;
        }

        Ok(post)
    }

    /// Delete a post an deliver the deletion request
    ///
    /// # Panics
    ///
    /// This should never ever panic. If it does, open a bug report.
    pub async fn delete(&self, delete_post: DeletePost) -> Result<()> {
        let Some(post) = Posts::find_by_id(delete_post.post_id)
            .one(&self.db_conn)
            .await?
        else {
            return Err(ApiError::NotFound.into());
        };

        if post.account_id != delete_post.account_id {
            if let Some(user_id) = delete_post.user_id {
                let admin_role_count = UsersRoles::find()
                    .filter(users_roles::Column::UserId.eq(user_id))
                    .filter(users_roles::Column::Role.eq(Role::Administrator))
                    .count(&self.db_conn)
                    .await?;

                if admin_role_count == 0 {
                    return Err(ApiError::Unauthorised.into());
                }
            } else {
                return Err(ApiError::Unauthorised.into());
            }
        }

        let job_context = Job::DeliverDelete(DeleteDeliveryContext { post_id: post.id });
        jobs::Model {
            id: Uuid::now_v7(),
            state: JobState::Queued,
            run_at: Utc::now().into(),
            context: serde_json::to_value(job_context).unwrap(),
            fail_count: 0,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }
        .into_active_model()
        .insert(&self.db_conn)
        .await?;

        self.search_service.remove_from_index(post.into()).await?;

        Ok(())
    }

    /// Favourite a post
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, create a bug report.
    pub async fn favourite(
        &self,
        post_id: Uuid,
        favouriting_account_id: Uuid,
    ) -> Result<posts::Model> {
        let post_query = Posts::find_by_id(post_id);
        let Some(post) = add_permission_checks(post_query, Some(favouriting_account_id))
            .one(&self.db_conn)
            .await?
        else {
            return Err(ApiError::NotFound.into());
        };

        let id = Uuid::now_v7();
        let url = format!("https://{}/favourites/{id}", self.config.domain);
        let favourite = favourites::Model {
            id,
            account_id: favouriting_account_id,
            post_id: post.id,
            url,
            created_at: Utc::now().into(),
        }
        .into_active_model()
        .insert(&self.db_conn)
        .await?;

        let context = Job::DeliverFavourite(FavouriteDeliveryContext {
            favourite_id: favourite.id,
        });

        jobs::Model {
            id: Uuid::now_v7(),
            state: JobState::Queued,
            run_at: Utc::now().into(),
            context: serde_json::to_value(context).unwrap(),
            fail_count: 0,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }
        .into_active_model()
        .insert(&self.db_conn)
        .await?;

        Ok(post)
    }

    /// Get a service by its ID
    ///
    /// Does checks whether the user is allowed to fetch the post
    pub async fn get_by_id(
        &self,
        id: Uuid,
        fetching_account_id: Option<Uuid>,
    ) -> Result<Option<posts::Model>> {
        let post_query = Posts::find_by_id(id);
        let post_query = add_permission_checks(post_query, fetching_account_id);

        post_query.one(&self.db_conn).await.map_err(Error::from)
    }
}

fn add_permission_checks(
    mut select_query: Select<Posts>,
    fetching_account_id: Option<Uuid>,
) -> Select<Posts> {
    let mut post_filter = posts::Column::Visibility
        .eq(Visibility::Public)
        .or(posts::Column::Visibility.eq(Visibility::Unlisted));

    if let Some(fetching_account_id) = fetching_account_id {
        // The post is owned by the user
        post_filter = post_filter.or(posts::Column::AccountId.eq(fetching_account_id));

        // Post is follower-only, and the user is following the author
        select_query = select_query.join(
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
                                    .filter(accounts_followers::Column::ApprovedAt.is_not_null())
                                    .select_only()
                                    .column(accounts_followers::Column::AccountId)
                                    .into_query(),
                            ),
                        )
                        .into_condition()
                }),
        );

        // Post is mention-only, and user is mentioned in the post
        select_query = select_query.join(
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

    select_query.filter(post_filter)
}
