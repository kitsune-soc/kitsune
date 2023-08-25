use super::{
    instance::InstanceService,
    job::{Enqueue, JobService},
    url::UrlService,
};
use crate::{
    error::{ApiError, Error, Result},
    event::{post::EventType, PostEvent, PostEventEmitter},
    job::deliver::{
        create::DeliverCreate,
        delete::DeliverDelete,
        favourite::DeliverFavourite,
        unfavourite::DeliverUnfavourite,
        update::{DeliverUpdate, UpdateEntity},
    },
    resolve::PostResolver,
    sanitize::CleanHtmlExt,
    util::process_markdown,
};
use async_stream::try_stream;
use derive_builder::Builder;
use diesel::{
    BelongingToDsl, BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl,
    SelectableHelper,
};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncPgConnection, RunQueryDsl};
use futures_util::{stream::BoxStream, Stream, StreamExt};
use garde::Validate;
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    model::{
        favourite::{Favourite, NewFavourite},
        media_attachment::NewPostMediaAttachment,
        mention::NewMention,
        post::{NewPost, Post, PostChangeset, Visibility},
        user_role::Role,
    },
    post_permission_check::{PermissionCheck, PostPermissionCheckExt},
    schema::{
        media_attachments, posts, posts_favourites, posts_media_attachments, posts_mentions,
        users_roles,
    },
    PgPool,
};
use kitsune_embed::Client as EmbedClient;
use kitsune_language::{DetectionBackend, Language};
use kitsune_search::{SearchBackend, SearchService};
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

pub struct PostValidationContext {
    character_limit: usize,
}

#[derive(Builder, Clone, Validate)]
#[garde(context(PostValidationContext as ctx))]
pub struct CreatePost {
    /// ID of the author
    ///
    /// This is not validated. Make sure this is a valid and verified value.
    #[garde(skip)]
    author_id: Uuid,

    /// ID of the post this post is replying to
    ///
    /// This is validated. If you pass in an non-existent ID, it will be ignored.
    #[builder(default, setter(strip_option))]
    #[garde(skip)]
    in_reply_to_id: Option<Uuid>,

    /// IDs of the media attachments attached to this post
    ///
    /// These IDs are validated. If one of them doesn't exist, the post is rejected.
    #[builder(default)]
    #[garde(skip)]
    media_ids: Vec<Uuid>,

    /// Mark this post as sensitive
    ///
    /// Defaults to false
    #[builder(default)]
    #[garde(skip)]
    sensitive: bool,

    /// Subject of the post
    ///
    /// This is optional
    #[builder(default, setter(strip_option))]
    #[garde(
        length(
            min = 1,
            max = ctx.character_limit.saturating_sub(
                content.chars().count()
            )
        )
    )]
    subject: Option<String>,

    /// Content of the post
    #[garde(
        length(
            min = 1,
            max = ctx.character_limit.saturating_sub(
                subject.as_ref().map_or(0, |subject| subject.chars().count())
            )
        )
    )]
    content: String,

    /// Process the content as a markdown document
    ///
    /// Defaults to true
    #[builder(default = "true")]
    #[garde(skip)]
    process_markdown: bool,

    /// Visibility of the post
    ///
    /// Defaults to public
    #[builder(default = "Visibility::Public")]
    #[garde(skip)]
    visibility: Visibility,

    /// ISO 639 language code of the post
    ///
    /// This is optional
    #[builder(default, setter(strip_option))]
    #[garde(skip)]
    language: Option<String>,
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

#[derive(Builder, Clone, Validate)]
#[garde(context(PostValidationContext as ctx))]
pub struct UpdatePost {
    /// ID of the post that is supposed to be updated
    #[garde(skip)]
    post_id: Uuid,

    /// IDs of the media attachments attached to this post
    ///
    /// These IDs are validated. If one of them doesn't exist, the post is rejected.
    #[builder(default)]
    #[garde(skip)]
    media_ids: Vec<Uuid>,

    /// Mark this post as sensitive
    ///
    /// Defaults to false
    #[builder(default)]
    #[garde(skip)]
    sensitive: Option<bool>,

    /// Subject of the post
    ///
    /// This is optional
    #[builder(default)]
    #[garde(
        length(
            min = 1,
            max = ctx.character_limit.saturating_sub(
                content.as_ref().map_or(0, |content| content.chars().count())
            )
        )
    )]
    subject: Option<String>,

    /// Content of the post
    #[builder(default)]
    #[garde(
        length(
            min = 1,
            max = ctx.character_limit.saturating_sub(
                subject.as_ref().map_or(0, |subject| subject.chars().count())
            )
        )
    )]
    content: Option<String>,

    /// Process the content as a markdown document
    ///
    /// Defaults to true
    #[builder(default = "true")]
    #[garde(skip)]
    process_markdown: bool,

    /// ISO 639 language code of the post
    ///
    /// This is optional
    #[builder(default, setter(strip_option))]
    #[garde(skip)]
    language: Option<String>,
}

impl UpdatePost {
    #[must_use]
    pub fn builder() -> UpdatePostBuilder {
        UpdatePostBuilder::default()
    }
}

#[derive(Clone, Builder)]
pub struct RepostPost {
    /// ID of the account that reposts the post
    account_id: Uuid,

    /// ID of the post that is supposed to be reposted
    post_id: Uuid,

    /// Visibility of the repost
    ///
    /// Defaults to Public
    #[builder(default = "Visibility::Public")]
    visibility: Visibility,
}

impl RepostPost {
    #[must_use]
    pub fn builder() -> RepostPostBuilder {
        RepostPostBuilder::default()
    }
}

#[derive(Clone, Builder)]
pub struct UnrepostPost {
    /// ID of the account that is associated with the user
    account_id: Uuid,

    /// ID of the post that is supposed to be unreposted
    post_id: Uuid,
}

impl UnrepostPost {
    #[must_use]
    pub fn builder() -> UnrepostPostBuilder {
        UnrepostPostBuilder::default()
    }
}

#[derive(Clone, TypedBuilder)]
pub struct PostService {
    db_pool: PgPool,
    embed_client: Option<EmbedClient>,
    instance_service: InstanceService,
    job_service: JobService,
    post_resolver: PostResolver,
    search_service: SearchService,
    status_event_emitter: PostEventEmitter,
    url_service: UrlService,
}

impl PostService {
    async fn process_media_attachments(
        conn: &mut AsyncPgConnection,
        post_id: Uuid,
        media_attachment_ids: &[Uuid],
    ) -> Result<()> {
        if media_attachment_ids.is_empty() {
            return Ok(());
        }

        if media_attachments::table
            .filter(media_attachments::id.eq_any(media_attachment_ids))
            .count()
            .get_result::<i64>(conn)
            .await?
            != media_attachment_ids.len() as i64
        {
            return Err(ApiError::BadRequest.into());
        }

        diesel::insert_into(posts_media_attachments::table)
            .values(
                media_attachment_ids
                    .iter()
                    .map(|media_id| NewPostMediaAttachment {
                        post_id,
                        media_attachment_id: *media_id,
                    })
                    .collect::<Vec<NewPostMediaAttachment>>(),
            )
            .execute(conn)
            .await?;

        Ok(())
    }

    async fn process_mentions(
        conn: &mut AsyncPgConnection,
        post_id: Uuid,
        mentioned_account_ids: Vec<(Uuid, String)>,
    ) -> Result<()> {
        if mentioned_account_ids.is_empty() {
            return Ok(());
        }

        diesel::insert_into(posts_mentions::table)
            .values(
                mentioned_account_ids
                    .iter()
                    .map(|(account_id, mention_text)| NewMention {
                        post_id,
                        account_id: *account_id,
                        mention_text,
                    })
                    .collect::<Vec<NewMention<'_>>>(),
            )
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;

        Ok(())
    }

    /// Create a new post and deliver it to the followers
    ///
    /// # Panics
    ///
    /// This should never ever panic. If it does, create a bug report.
    pub async fn create(&self, create_post: CreatePost) -> Result<Post> {
        create_post.validate(&PostValidationContext {
            character_limit: self.instance_service.character_limit(),
        })?;

        let subject = create_post.subject.map(|mut subject| {
            subject.clean_html();
            subject
        });

        let mut content = if create_post.process_markdown {
            process_markdown(&create_post.content)
        } else {
            create_post.content
        };
        content.clean_html();

        let detect_language =
            |s: &str| kitsune_language::detect_language(DetectionBackend::default(), s);
        let content_lang = create_post.language.map_or_else(
            || detect_language(&content),
            |lang| Language::from_639_1(&lang).unwrap_or_else(|| detect_language(&content)),
        );

        let (mentioned_account_ids, content) = self.post_resolver.resolve(&content).await?;
        let link_preview_url = if let Some(ref embed_client) = self.embed_client {
            embed_client
                .fetch_embed_for_fragment(&content)
                .await?
                .map(|fragment_embed| fragment_embed.url)
        } else {
            None
        };

        let id = Uuid::now_v7();
        let url = self.url_service.post_url(id);

        let post = self
            .db_pool
            .with_transaction(move |tx| {
                async move {
                    let in_reply_to_id = if let Some(in_reply_to_id) = create_post.in_reply_to_id {
                        (posts::table
                            .find(in_reply_to_id)
                            .count()
                            .get_result::<i64>(tx)
                            .await?
                            != 0)
                            .then_some(in_reply_to_id)
                    } else {
                        None
                    };

                    let post: Post = diesel::insert_into(posts::table)
                        .values(NewPost {
                            id,
                            account_id: create_post.author_id,
                            in_reply_to_id,
                            reposted_post_id: None,
                            subject: subject.as_deref(),
                            content: content.as_str(),
                            content_lang: content_lang.into(),
                            link_preview_url: link_preview_url.as_deref(),
                            is_sensitive: create_post.sensitive,
                            visibility: create_post.visibility,
                            is_local: true,
                            url: url.as_str(),
                            created_at: None,
                        })
                        .returning(Post::as_returning())
                        .get_result(tx)
                        .await?;

                    Self::process_mentions(tx, post.id, mentioned_account_ids).await?;
                    Self::process_media_attachments(tx, post.id, &create_post.media_ids).await?;

                    Ok::<_, Error>(post)
                }
                .scope_boxed()
            })
            .await?;

        self.job_service
            .enqueue(
                Enqueue::builder()
                    .job(DeliverCreate { post_id: post.id })
                    .build(),
            )
            .await?;

        if post.visibility == Visibility::Public || post.visibility == Visibility::Unlisted {
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
        let post: Post = self
            .db_pool
            .with_connection(|mut db_conn| {
                posts::table
                    .find(delete_post.post_id)
                    .select(Post::as_select())
                    .first(&mut db_conn)
            })
            .await?;

        if post.account_id != delete_post.account_id {
            if let Some(user_id) = delete_post.user_id {
                let admin_role_count = self
                    .db_pool
                    .with_connection(|mut db_conn| {
                        users_roles::table
                            .filter(
                                users_roles::user_id
                                    .eq(user_id)
                                    .and(users_roles::role.eq(Role::Administrator)),
                            )
                            .count()
                            .get_result::<i64>(&mut db_conn)
                    })
                    .await?;

                if admin_role_count == 0 {
                    return Err(ApiError::Unauthorised.into());
                }
            } else {
                return Err(ApiError::Unauthorised.into());
            }
        }

        self.job_service
            .enqueue(
                Enqueue::builder()
                    .job(DeliverDelete { post_id: post.id })
                    .build(),
            )
            .await?;

        self.status_event_emitter
            .emit(PostEvent {
                r#type: EventType::Delete,
                post_id: post.id,
            })
            .await
            .map_err(Error::Event)?;

        self.search_service.remove_from_index(&post.into()).await?;

        Ok(())
    }

    /// Update a post and deliver the update
    ///
    /// # Panics
    ///
    /// This should never ever panic. If it does, create a bug report.
    pub async fn update(&self, update_post: UpdatePost) -> Result<Post> {
        update_post.validate(&PostValidationContext {
            character_limit: self.instance_service.character_limit(),
        })?;

        let subject = update_post.subject.map(|mut subject| {
            subject.clean_html();
            subject
        });

        let mut content = if update_post.process_markdown {
            update_post.content.as_ref().map(|s| process_markdown(s))
        } else {
            update_post.content
        };
        if let Some(content) = &mut content {
            content.clean_html();
        };

        // If a new language code was submitted, we should update the post language accordingly
        // If the language code is not provided, only the updated body, perform language detection normally
        // Otherwise, don't update anything
        let content_lang = match update_post.language {
            Some(lang) => Language::from_639_1(&lang),
            None => content
                .as_ref()
                .map(|c| kitsune_language::detect_language(DetectionBackend::default(), c)),
        };

        let (mentioned_account_ids, content) = match content.as_ref() {
            Some(content) => {
                let resolved = self.post_resolver.resolve(content).await?;
                (resolved.0, Some(resolved.1))
            }
            None => (Vec::new(), None),
        };

        let link_preview_url = if let (Some(embed_client), Some(content)) =
            (self.embed_client.as_ref(), content.as_ref())
        {
            embed_client
                .fetch_embed_for_fragment(content)
                .await?
                .map(|fragment_embed| fragment_embed.url)
        } else {
            None
        };

        let post = self
            .db_pool
            .with_transaction(move |tx| {
                async move {
                    let post: Post = diesel::update(posts::table)
                        .set(PostChangeset {
                            id: update_post.post_id,
                            subject: subject.as_deref(),
                            content: content.as_deref(),
                            content_lang: content_lang.map(Into::into),
                            link_preview_url: link_preview_url.as_deref(),
                            is_sensitive: update_post.sensitive,
                            updated_at: Timestamp::now_utc(),
                        })
                        .returning(Post::as_returning())
                        .get_result(tx)
                        .await?;

                    Self::process_mentions(tx, post.id, mentioned_account_ids).await?;
                    Self::process_media_attachments(tx, post.id, &update_post.media_ids).await?;

                    Ok::<_, Error>(post)
                }
                .scope_boxed()
            })
            .await?;

        self.job_service
            .enqueue(
                Enqueue::builder()
                    .job(DeliverUpdate {
                        entity: UpdateEntity::Status,
                        id: post.id,
                    })
                    .build(),
            )
            .await?;

        if post.visibility == Visibility::Public || post.visibility == Visibility::Unlisted {
            self.search_service
                .update_in_index(post.clone().into())
                .await?;
        }

        self.status_event_emitter
            .emit(PostEvent {
                r#type: EventType::Update,
                post_id: post.id,
            })
            .await
            .map_err(Error::Event)?;

        Ok(post)
    }

    /// Repost a post
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, create a bug report.
    pub async fn repost(&self, repost_post: RepostPost) -> Result<Post> {
        let permission_check = PermissionCheck::builder()
            .fetching_account_id(Some(repost_post.account_id))
            .build()
            .unwrap();

        let existing_repost: Option<Post> = self
            .db_pool
            .with_connection(|mut db_conn| async move {
                posts::table
                    .filter(
                        posts::reposted_post_id
                            .eq(repost_post.post_id)
                            .and(posts::account_id.eq(repost_post.account_id)),
                    )
                    .add_post_permission_check(permission_check)
                    .select(Post::as_select())
                    .first(&mut db_conn)
                    .await
                    .optional()
            })
            .await?;

        if let Some(repost) = existing_repost {
            return Ok(repost);
        }

        let post: Post = self
            .db_pool
            .with_connection(|mut db_conn| {
                posts::table
                    .find(repost_post.post_id)
                    .add_post_permission_check(permission_check)
                    .select(Post::as_select())
                    .get_result(&mut db_conn)
            })
            .await?;

        let id = Uuid::now_v7();
        let url = self.url_service.post_url(id);

        let repost = self
            .db_pool
            .with_connection(|mut db_conn| {
                diesel::insert_into(posts::table)
                    .values(NewPost {
                        id,
                        account_id: repost_post.account_id,
                        in_reply_to_id: None,
                        reposted_post_id: Some(post.id),
                        subject: Some(""),
                        content: "",
                        content_lang: post.content_lang,
                        link_preview_url: None,
                        is_sensitive: post.is_sensitive,
                        visibility: repost_post.visibility,
                        is_local: true,
                        url: url.as_str(),
                        created_at: Some(Timestamp::now_utc()),
                    })
                    .returning(Post::as_returning())
                    .get_result(&mut db_conn)
            })
            .await?;

        self.job_service
            .enqueue(
                Enqueue::builder()
                    .job(DeliverCreate { post_id: repost.id })
                    .build(),
            )
            .await?;

        self.status_event_emitter
            .emit(PostEvent {
                r#type: EventType::Create,
                post_id: repost.id,
            })
            .await
            .map_err(Error::Event)?;

        Ok(repost)
    }

    /// Unrepost a post
    ///
    /// # Panics
    ///
    /// This should never ever panic. If it does, open a bug report.
    pub async fn unrepost(&self, unrepost_post: UnrepostPost) -> Result<Post> {
        let permission_check = PermissionCheck::builder()
            .fetching_account_id(Some(unrepost_post.account_id))
            .build()
            .unwrap();

        let post: Post = self
            .db_pool
            .with_connection(|mut db_conn| {
                posts::table
                    .filter(
                        posts::account_id
                            .eq(unrepost_post.account_id)
                            .and(posts::reposted_post_id.eq(unrepost_post.post_id)),
                    )
                    .add_post_permission_check(permission_check)
                    .select(Post::as_select())
                    .first(&mut db_conn)
            })
            .await?;

        self.job_service
            .enqueue(
                Enqueue::builder()
                    .job(DeliverDelete { post_id: post.id })
                    .build(),
            )
            .await?;

        self.status_event_emitter
            .emit(PostEvent {
                r#type: EventType::Delete,
                post_id: post.id,
            })
            .await
            .map_err(Error::Event)?;

        Ok(post)
    }

    /// Favourite a post
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, create a bug report.
    pub async fn favourite(&self, post_id: Uuid, favouriting_account_id: Uuid) -> Result<Post> {
        let permission_check = PermissionCheck::builder()
            .fetching_account_id(Some(favouriting_account_id))
            .build()
            .unwrap();

        let post: Post = self
            .db_pool
            .with_connection(|mut db_conn| {
                posts::table
                    .find(post_id)
                    .add_post_permission_check(permission_check)
                    .select(Post::as_select())
                    .get_result(&mut db_conn)
            })
            .await?;

        let id = Uuid::now_v7();
        let url = self.url_service.favourite_url(id);
        let favourite_id = self
            .db_pool
            .with_connection(|mut db_conn| {
                diesel::insert_into(posts_favourites::table)
                    .values(NewFavourite {
                        id,
                        account_id: favouriting_account_id,
                        post_id: post.id,
                        url,
                        created_at: None,
                    })
                    .returning(posts_favourites::id)
                    .get_result(&mut db_conn)
            })
            .await?;

        self.job_service
            .enqueue(
                Enqueue::builder()
                    .job(DeliverFavourite { favourite_id })
                    .build(),
            )
            .await?;

        Ok(post)
    }

    /// Unfavourite a post
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, please open a bug report.
    pub async fn unfavourite(&self, post_id: Uuid, favouriting_account_id: Uuid) -> Result<Post> {
        let post = self
            .get_by_id(post_id, Some(favouriting_account_id))
            .await?;

        let favourite = self
            .db_pool
            .with_connection(|mut db_conn| {
                let post = &post;

                async move {
                    Favourite::belonging_to(&post)
                        .filter(posts_favourites::account_id.eq(favouriting_account_id))
                        .get_result::<Favourite>(&mut db_conn)
                        .await
                        .optional()
                }
            })
            .await?;

        if let Some(favourite) = favourite {
            self.job_service
                .enqueue(
                    Enqueue::builder()
                        .job(DeliverUnfavourite {
                            favourite_id: favourite.id,
                        })
                        .build(),
                )
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
    pub async fn get_by_id(&self, id: Uuid, fetching_account_id: Option<Uuid>) -> Result<Post> {
        let permission_check = PermissionCheck::builder()
            .fetching_account_id(fetching_account_id)
            .build()
            .unwrap();

        self.db_pool
            .with_connection(|mut db_conn| {
                posts::table
                    .find(id)
                    .add_post_permission_check(permission_check)
                    .select(Post::as_select())
                    .get_result(&mut db_conn)
            })
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
    ) -> impl Stream<Item = Result<Post>> + '_ {
        try_stream! {
            let mut last_post = self.get_by_id(id, fetching_account_id).await?;
            let permission_check = PermissionCheck::builder()
                .fetching_account_id(fetching_account_id)
                .build()
                .unwrap();

            while let Some(in_reply_to_id) = last_post.in_reply_to_id {
                let post = self.db_pool
                    .with_connection(|mut db_conn| {
                        posts::table
                            .find(in_reply_to_id)
                            .add_post_permission_check(permission_check)
                            .select(Post::as_select())
                            .get_result::<Post>(&mut db_conn)
                    })
                    .await?;

                yield post.clone();

                last_post = post;
            }
        }
    }

    /// Get the descendants of the post
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, please open an issue.
    #[must_use]
    pub fn get_descendants(
        &self,
        id: Uuid,
        fetching_account_id: Option<Uuid>,
    ) -> BoxStream<'_, Result<Post>> {
        try_stream! {
            let permission_check = PermissionCheck::builder()
                .fetching_account_id(fetching_account_id)
                .build()
                .unwrap();

            let descendant_stream = self.db_pool
                .with_connection(|mut db_conn| {
                    posts::table
                        .filter(posts::in_reply_to_id.eq(id))
                        .add_post_permission_check(permission_check)
                        .select(Post::as_select())
                        .load_stream::<Post>(&mut db_conn)
                })
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

#[cfg(test)]
mod test {
    use crate::service::post::{CreatePost, PostValidationContext, UpdatePost};
    use garde::Validate;
    use speedy_uuid::Uuid;

    #[test]
    fn new_post_character_limit() {
        let create_post = CreatePost::builder()
            .author_id(Uuid::now_v7())
            .subject("hello".into())
            .content("world".into())
            .build()
            .unwrap();

        assert!(create_post
            .validate(&PostValidationContext {
                character_limit: 20,
            })
            .is_ok());

        assert!(create_post
            .validate(&PostValidationContext { character_limit: 5 })
            .is_err());

        assert!(create_post
            .validate(&PostValidationContext { character_limit: 2 })
            .is_err());
    }

    #[test]
    fn update_post_character_limit() {
        let update_post = UpdatePost::builder()
            .post_id(Uuid::now_v7())
            .subject(Some("hello".into()))
            .content(Some("world".into()))
            .build()
            .unwrap();

        assert!(update_post
            .validate(&PostValidationContext {
                character_limit: 20,
            })
            .is_ok());

        assert!(update_post
            .validate(&PostValidationContext { character_limit: 5 })
            .is_err());

        assert!(update_post
            .validate(&PostValidationContext { character_limit: 2 })
            .is_err());
    }
}
