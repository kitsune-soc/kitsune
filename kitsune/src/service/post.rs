use super::{
    instance::InstanceService,
    job::{Enqueue, JobService},
    url::UrlService,
};
use crate::{
    error::{ApiError, Error, Result},
    event::{post::EventType, PostEvent, PostEventEmitter},
    job::deliver::{
        create::DeliverCreate, delete::DeliverDelete, favourite::DeliverFavourite,
        unfavourite::DeliverUnfavourite,
    },
    resolve::PostResolver,
    sanitize::CleanHtmlExt,
};
use async_stream::try_stream;
use derive_builder::Builder;
use diesel::{
    BelongingToDsl, BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl,
    SelectableHelper,
};
use diesel_async::{
    scoped_futures::ScopedFutureExt, AsyncConnection, AsyncPgConnection, RunQueryDsl,
};
use futures_util::{stream::BoxStream, Stream, StreamExt};
use kitsune_db::{
    model::{
        favourite::{Favourite, NewFavourite},
        media_attachment::NewPostMediaAttachment,
        mention::NewMention,
        post::{NewPost, Post, Visibility},
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
use kitsune_search::{SearchBackend, SearchService};
use pulldown_cmark::{html, Options, Parser};
use typed_builder::TypedBuilder;
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

    /// IDs of the media attachments attached to this post
    ///
    /// These IDs are validated. If one of them doesn't exist, the post is rejected.
    #[builder(default)]
    media_ids: Vec<Uuid>,

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

#[derive(Clone, TypedBuilder)]
pub struct PostService {
    db_conn: PgPool,
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
        if create_post.content.chars().count() > self.instance_service.character_limit() {
            return Err(ApiError::BadRequest.into());
        }

        let subject = create_post.subject.map(|mut subject| {
            subject.clean_html();
            subject
        });
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

        let mut db_conn = self.db_conn.get().await?;
        let post = db_conn
            .transaction(move |tx| {
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
        let mut db_conn = self.db_conn.get().await?;
        let post: Post = posts::table
            .find(delete_post.post_id)
            .select(Post::as_select())
            .first(&mut db_conn)
            .await?;

        if post.account_id != delete_post.account_id {
            if let Some(user_id) = delete_post.user_id {
                let admin_role_count = users_roles::table
                    .filter(
                        users_roles::user_id
                            .eq(user_id)
                            .and(users_roles::role.eq(Role::Administrator)),
                    )
                    .count()
                    .get_result::<i64>(&mut db_conn)
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

        self.search_service.remove_from_index(post.into()).await?;

        Ok(())
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

        let mut db_conn = self.db_conn.get().await?;
        let post: Post = posts::table
            .find(post_id)
            .add_post_permission_check(permission_check)
            .select(Post::as_select())
            .get_result(&mut db_conn)
            .await?;

        let id = Uuid::now_v7();
        let url = self.url_service.favourite_url(id);
        let favourite_id = diesel::insert_into(posts_favourites::table)
            .values(NewFavourite {
                id,
                account_id: favouriting_account_id,
                post_id: post.id,
                url,
                created_at: None,
            })
            .returning(posts_favourites::id)
            .get_result(&mut db_conn)
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

        let mut db_conn = self.db_conn.get().await?;
        if let Some(favourite) = Favourite::belonging_to(&post)
            .filter(posts_favourites::account_id.eq(favouriting_account_id))
            .get_result::<Favourite>(&mut db_conn)
            .await
            .optional()?
        {
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
        let mut db_conn = self.db_conn.get().await?;
        let permission_check = PermissionCheck::builder()
            .fetching_account_id(fetching_account_id)
            .build()
            .unwrap();

        posts::table
            .find(id)
            .add_post_permission_check(permission_check)
            .select(Post::as_select())
            .get_result(&mut db_conn)
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
                let mut db_conn = self.db_conn.get().await?;
                let post = posts::table
                    .find(in_reply_to_id)
                    .add_post_permission_check(permission_check)
                    .select(Post::as_select())
                    .get_result::<Post>(&mut db_conn)
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

            let mut db_conn = self.db_conn.get().await?;
            let descendant_stream = posts::table
                .filter(posts::in_reply_to_id.eq(id))
                .add_post_permission_check(permission_check)
                .select(Post::as_select())
                .load_stream::<Post>(&mut db_conn)
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
