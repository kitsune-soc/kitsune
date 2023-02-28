use super::search::SearchService;
use crate::{
    cache::Cache,
    config::Configuration,
    error::{ApiError, Error, Result},
    event::{post::EventType, PostEvent, PostEventEmitter},
    job::{
        deliver::{
            create::CreateDeliveryContext, delete::DeleteDeliveryContext,
            favourite::FavouriteDeliveryContext, unfavourite::UnfavouriteDeliveryContext,
        },
        Job,
    },
    resolve::PostResolver,
    sanitize::CleanHtmlExt,
};
use async_stream::try_stream;
use chrono::Utc;
use derive_builder::Builder;
use futures_util::{stream::BoxStream, FutureExt, Stream, StreamExt};
use kitsune_db::{
    custom::{JobState, Role, Visibility},
    entity::{
        accounts, favourites, jobs, posts, posts_mentions,
        prelude::{Favourites, Jobs, Posts, PostsMentions, UsersRoles},
        users_roles,
    },
    link::InReplyTo,
    r#trait::{PermissionCheck, PostPermissionCheckExt},
};
use pulldown_cmark::{html, Options, Parser};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait,
    PaginatorTrait, QueryFilter, TransactionTrait,
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

    /// Process the content as a markdown document
    ///
    /// Defaults to true
    #[builder(default = "true")]
    process_markdown: bool,

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
    post_resolver: PostResolver<S, FPC, FUC, WC>,
    search_service: S,
    status_event_emitter: PostEventEmitter,
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
        let mut content = if create_post.process_markdown {
            let parser = Parser::new_ext(&create_post.content, Options::all());
            let mut buf = String::new();
            html::push_html(&mut buf, parser);
            buf
        } else {
            create_post.content
        };
        content.clean_html();

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
                        reposted_post_id: None,
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
                        PostsMentions::insert(
                            posts_mentions::Model {
                                account_id,
                                post_id: post.id,
                            }
                            .into_active_model(),
                        )
                        .exec_without_returning(tx)
                        .await?;
                    }

                    let job_context =
                        Job::DeliverCreate(CreateDeliveryContext { post_id: post.id });

                    Jobs::insert(
                        jobs::Model {
                            id: Uuid::now_v7(),
                            state: JobState::Queued,
                            run_at: Utc::now().into(),
                            context: serde_json::to_value(job_context).unwrap(),
                            fail_count: 0,
                            created_at: Utc::now().into(),
                            updated_at: Utc::now().into(),
                        }
                        .into_active_model(),
                    )
                    .exec_without_returning(tx)
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

        self.status_event_emitter
            .emit(PostEvent {
                r#type: EventType::Create,
                post_id: post.id,
            })
            .await
            .map_err(Error::Event)?;

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
        Jobs::insert(
            jobs::Model {
                id: Uuid::now_v7(),
                state: JobState::Queued,
                run_at: Utc::now().into(),
                context: serde_json::to_value(job_context).unwrap(),
                fail_count: 0,
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
            }
            .into_active_model(),
        )
        .exec_without_returning(&self.db_conn)
        .await?;

        self.status_event_emitter
            .emit(PostEvent {
                r#type: EventType::Delete,
                post_id: post.id,
            })
            .await
            .map_err(Error::Event)?;

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
        let permission_check = PermissionCheck::builder()
            .fetching_account_id(Some(favouriting_account_id))
            .build()
            .unwrap();

        let Some(post) = Posts::find_by_id(post_id)
            .add_permission_checks(permission_check)
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

        Jobs::insert(
            jobs::Model {
                id: Uuid::now_v7(),
                state: JobState::Queued,
                run_at: Utc::now().into(),
                context: serde_json::to_value(context).unwrap(),
                fail_count: 0,
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
            }
            .into_active_model(),
        )
        .exec_without_returning(&self.db_conn)
        .await?;

        Ok(post)
    }

    /// Unfavourite a post
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, please open a bug report.
    pub async fn unfavourite(
        &self,
        post_id: Uuid,
        favouriting_account_id: Uuid,
    ) -> Result<posts::Model> {
        let Some(post) = self
            .get_by_id(post_id, Some(favouriting_account_id))
            .await?
        else {
            return Err(ApiError::NotFound.into());
        };

        if let Some(favourite) = post
            .find_related(Favourites)
            .filter(favourites::Column::AccountId.eq(favouriting_account_id))
            .one(&self.db_conn)
            .await?
        {
            let context = Job::DeliverUnfavourite(UnfavouriteDeliveryContext {
                favourite_id: favourite.id,
            });

            Jobs::insert(
                jobs::Model {
                    id: Uuid::now_v7(),
                    state: JobState::Queued,
                    run_at: Utc::now().into(),
                    context: serde_json::to_value(context).unwrap(),
                    fail_count: 0,
                    created_at: Utc::now().into(),
                    updated_at: Utc::now().into(),
                }
                .into_active_model(),
            )
            .exec_without_returning(&self.db_conn)
            .await?;
        }

        Ok(post)
    }

    /// Get a post by its ID
    ///
    /// Does checks whether the user is allowed to fetch the post
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, please open an issue.
    pub async fn get_by_id(
        &self,
        id: Uuid,
        fetching_account_id: Option<Uuid>,
    ) -> Result<Option<posts::Model>> {
        let permission_check = PermissionCheck::builder()
            .fetching_account_id(fetching_account_id)
            .build()
            .unwrap();

        Posts::find_by_id(id)
            .add_permission_checks(permission_check)
            .one(&self.db_conn)
            .await
            .map_err(Error::from)
    }

    /// Get the ancestors of the post
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, please open an issue.
    pub fn get_ancestors(
        &self,
        id: Uuid,
        fetching_account_id: Option<Uuid>,
    ) -> impl Stream<Item = Result<posts::Model>> + '_ {
        try_stream! {
            let mut last_post = self.get_by_id(id, fetching_account_id).await?;
            let permission_check = PermissionCheck::builder()
                .fetching_account_id(fetching_account_id)
                .build()
                .unwrap();

            while let Some(post) = last_post.take() {
                let post = post
                    .find_linked(InReplyTo)
                    .add_permission_checks(permission_check.clone())
                    .one(&self.db_conn)
                    .await?;

                if let Some(ref post) = post {
                    yield post.clone();
                }

                last_post = post;
            }
        }
    }

    /// Get the descendants of the post
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, please open an issue.
    pub fn get_descendants(
        &self,
        id: Uuid,
        fetching_account_id: Option<Uuid>,
    ) -> BoxStream<'_, Result<posts::Model>> {
        try_stream! {
            let permission_check = PermissionCheck::builder()
                .fetching_account_id(fetching_account_id)
                .build()
                .unwrap();

            let descendant_stream = Posts::find()
                .filter(posts::Column::InReplyToId.eq(id))
                .add_permission_checks(permission_check)
                .stream(&self.db_conn)
                .await?;

            for await descendant in descendant_stream {
                let descendant = descendant?;
                let descendant_id = descendant.id;

                yield descendant;

                let sub_descendants = self.get_descendants(descendant_id, fetching_account_id);
                for await sub_descendant in sub_descendants {
                    yield sub_descendant?;
                }
            }
        }
        .boxed()
    }
}
