use super::{
    instance::InstanceService,
    job::{Enqueue, JobService},
    notification::NotificationService,
    LimitContext,
};
use async_stream::try_stream;
use diesel::{
    BelongingToDsl, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension,
    QueryDsl, SelectableHelper,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures_util::{stream::BoxStream, Stream, StreamExt, TryStreamExt};
use garde::Validate;
use iso8601_timestamp::Timestamp;
use kitsune_config::language_detection::Configuration as LanguageDetectionConfig;
use kitsune_db::{
    model::{
        account::Account,
        custom_emoji::PostCustomEmoji,
        favourite::{Favourite, NewFavourite},
        media_attachment::NewPostMediaAttachment,
        mention::NewMention,
        notification::{NewNotification, Notification},
        post::{NewPost, PartialPostChangeset, Post, PostSource, Visibility},
        user_role::Role,
    },
    post_permission_check::{PermissionCheck, PostPermissionCheckExt},
    schema::{
        accounts, accounts_preferences, media_attachments, notifications, posts,
        posts_custom_emojis, posts_favourites, posts_media_attachments, posts_mentions,
        users_roles,
    },
    with_connection, with_transaction, PgPool,
};
use kitsune_embed::Client as EmbedClient;
use kitsune_error::{bail, Error, ErrorType, Result};
use kitsune_jobs::deliver::{
    create::DeliverCreate,
    delete::DeliverDelete,
    favourite::DeliverFavourite,
    unfavourite::DeliverUnfavourite,
    update::{DeliverUpdate, UpdateEntity},
};
use kitsune_language::Language;
use kitsune_search::SearchBackend;
use kitsune_url::UrlService;
use kitsune_util::{process, sanitize::CleanHtmlExt};
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

mod resolver;

pub use self::resolver::PostResolver;

macro_rules! min_character_limit {
    ($self:ident) => {{
        if $self.media_ids.is_empty() {
            1
        } else {
            0
        }
    }};
}

pub struct PostValidationContext {
    character_limit: usize,
}

impl PostValidationContext {
    fn max_character_limit(&self, other_value: usize) -> usize {
        // Saturating subtraction to prevent panics/wrapping
        self.character_limit.saturating_sub(other_value)
    }
}

#[derive(Clone, TypedBuilder, Validate)]
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
    #[builder(default)]
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
    #[builder(default)]
    #[garde(
        length(
            min = 1,
            max = ctx.max_character_limit(
                content.chars().count()
            )
        )
    )]
    subject: Option<String>,

    /// Content of the post
    #[garde(
        length(
            min = min_character_limit!(self),
            max = ctx.max_character_limit(
                subject.as_ref().map_or(0, |subject| subject.chars().count())
            )
        )
    )]
    content: String,

    /// Process the content as a markdown document
    ///
    /// Defaults to true
    #[builder(default = true)]
    #[garde(skip)]
    process_markdown: bool,

    /// Visibility of the post
    ///
    /// Defaults to public
    #[builder(default = Visibility::Public)]
    #[garde(skip)]
    visibility: Visibility,

    /// ISO 639 language code of the post
    ///
    /// This is optional
    #[builder(default, setter(strip_option))]
    #[garde(skip)]
    language: Option<String>,
}

#[derive(Clone, TypedBuilder)]
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

#[derive(Clone, TypedBuilder, Validate)]
#[garde(context(PostValidationContext as ctx))]
pub struct UpdatePost {
    /// ID of the post that is supposed to be updated
    #[garde(skip)]
    post_id: Uuid,

    /// ID of the account making the request
    #[garde(skip)]
    account_id: Uuid,

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
            max = ctx.max_character_limit(
                content.as_ref().map_or(0, |content| content.chars().count())
            )
        )
    )]
    subject: Option<String>,

    /// Content of the post
    #[builder(default)]
    #[garde(
        length(
            min = min_character_limit!(self),
            max = ctx.max_character_limit(
                subject.as_ref().map_or(0, |subject| subject.chars().count())
            )
        )
    )]
    content: Option<String>,

    /// Process the content as a markdown document
    ///
    /// Defaults to true
    #[builder(default = true)]
    #[garde(skip)]
    process_markdown: bool,

    /// ISO 639 language code of the post
    ///
    /// This is optional
    #[builder(default, setter(strip_option))]
    #[garde(skip)]
    language: Option<String>,
}

#[derive(Clone, TypedBuilder)]
pub struct RepostPost {
    /// ID of the account that reposts the post
    account_id: Uuid,

    /// ID of the post that is supposed to be reposted
    post_id: Uuid,

    /// Visibility of the repost
    ///
    /// Defaults to Public
    #[builder(default = Visibility::Public)]
    visibility: Visibility,
}

#[derive(Clone, TypedBuilder)]
pub struct UnrepostPost {
    /// ID of the account that is associated with the user
    account_id: Uuid,

    /// ID of the post that is supposed to be unreposted
    post_id: Uuid,
}

#[derive(Clone, TypedBuilder, Validate)]
#[garde(context(LimitContext as ctx))]
pub struct GetAccountsInteractingWithPost {
    /// ID of the account whose posts are getting fetched
    #[garde(skip)]
    post_id: Uuid,

    /// ID of the account that is requesting the posts
    #[builder(default)]
    #[garde(skip)]
    fetching_account_id: Option<Uuid>,

    /// Limit of returned posts
    #[garde(range(max = ctx.limit))]
    limit: usize,

    /// Smallest ID, return results starting from this ID
    ///
    /// Used for pagination
    #[builder(default)]
    #[garde(skip)]
    min_id: Option<Uuid>,

    /// Smallest ID, return highest results
    ///
    /// Used for pagination
    #[builder(default)]
    #[garde(skip)]
    since_id: Option<Uuid>,

    /// Largest ID
    ///
    /// Used for pagination
    #[builder(default)]
    #[garde(skip)]
    max_id: Option<Uuid>,
}

#[derive(Clone, TypedBuilder)]
pub struct PostService {
    db_pool: PgPool,
    embed_client: Option<EmbedClient>,
    instance_service: InstanceService,
    job_service: JobService,
    language_detection_config: LanguageDetectionConfig,
    post_resolver: PostResolver,
    search_backend: kitsune_search::AnySearchBackend,
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
            bail!(type = ErrorType::BadRequest(None), "tried to attach unknown attachment ids");
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
        author_id: Uuid,
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

        let accounts_to_notify: Vec<Uuid> = accounts::table
            .inner_join(accounts_preferences::table)
            .filter(
                accounts_preferences::account_id
                    .eq_any(
                        mentioned_account_ids
                            .iter()
                            .map(|(account_id, _)| account_id),
                    )
                    .and(accounts_preferences::notify_on_mention.eq(true))
                    .and(accounts::local.eq(true)),
            )
            .select(accounts::id)
            .load_stream::<Uuid>(conn)
            .await?
            .try_collect()
            .await?;

        diesel::insert_into(notifications::table)
            .values(
                accounts_to_notify
                    .iter()
                    .map(|acc| {
                        NewNotification::builder()
                            .receiving_account_id(*acc)
                            .mention(author_id, post_id)
                    })
                    .collect::<Vec<Notification>>(),
            )
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;

        Ok(())
    }

    async fn process_custom_emojis(
        conn: &mut AsyncPgConnection,
        post_id: Uuid,
        custom_emojis: Vec<(Uuid, String)>,
    ) -> Result<()> {
        if custom_emojis.is_empty() {
            return Ok(());
        }

        diesel::insert_into(posts_custom_emojis::table)
            .values(
                custom_emojis
                    .iter()
                    .map(|(emoji_id, emoji_text)| PostCustomEmoji {
                        post_id,
                        custom_emoji_id: *emoji_id,
                        emoji_text: emoji_text.to_string(),
                    })
                    .collect::<Vec<PostCustomEmoji>>(),
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
    #[allow(clippy::too_many_lines)]
    pub async fn create(&self, create_post: CreatePost) -> Result<Post> {
        create_post.validate(&PostValidationContext {
            character_limit: self.instance_service.character_limit(),
        })?;

        let subject = create_post.subject.map(|mut subject| {
            subject.clean_html();
            subject
        });

        let content_source = create_post.content.clone();
        let mut content = if create_post.process_markdown {
            process::markdown(&create_post.content)
        } else {
            create_post.content
        };

        content.clean_html();

        let detect_language =
            |s: &str| kitsune_language::detect_language(self.language_detection_config, s);
        let content_lang = create_post.language.map_or_else(
            || detect_language(&content),
            |lang| Language::from_639_1(&lang).unwrap_or_else(|| detect_language(&content)),
        );

        let resolved = self.post_resolver.resolve(&content).await?;
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

        let post = with_transaction!(self.db_pool, |tx| {
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
                    content: resolved.content.as_str(),
                    content_source: content_source.as_str(),
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

            Self::process_mentions(tx, post.account_id, post.id, resolved.mentioned_accounts)
                .await?;
            Self::process_custom_emojis(tx, post.id, resolved.custom_emojis).await?;
            Self::process_media_attachments(tx, post.id, &create_post.media_ids).await?;
            NotificationService::notify_on_new_post(tx, post.account_id, post.id).await?;

            Ok::<_, Error>(post)
        })?;

        self.job_service
            .enqueue(
                Enqueue::builder()
                    .job(DeliverCreate { post_id: post.id })
                    .build(),
            )
            .await?;

        if post.visibility == Visibility::Public || post.visibility == Visibility::Unlisted {
            self.search_backend
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
        let post = self
            .get_post_with_access_guard(
                delete_post.post_id,
                delete_post.account_id,
                delete_post.user_id,
            )
            .await?;

        self.job_service
            .enqueue(
                Enqueue::builder()
                    .job(DeliverDelete { post_id: post.id })
                    .build(),
            )
            .await?;

        self.search_backend.remove_from_index(&post.into()).await?;

        Ok(())
    }

    /// Update a post and deliver the update
    ///
    /// # Panics
    ///
    /// This should never ever panic. If it does, create a bug report.
    pub async fn update(&self, update_post: UpdatePost) -> Result<Post> {
        let _post = self
            .get_post_with_access_guard(update_post.post_id, update_post.account_id, None)
            .await?;

        update_post.validate(&PostValidationContext {
            character_limit: self.instance_service.character_limit(),
        })?;

        let subject = update_post.subject.map(|mut subject| {
            subject.clean_html();
            subject
        });

        let mut content = if update_post.process_markdown {
            update_post.content.as_deref().map(process::markdown)
        } else {
            update_post.content.clone()
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
                .map(|c| kitsune_language::detect_language(self.language_detection_config, c)),
        };

        let (mentioned_account_ids, custom_emojis, content) = match content.as_ref() {
            Some(content) => {
                let resolved = self.post_resolver.resolve(content).await?;
                (
                    resolved.mentioned_accounts,
                    resolved.custom_emojis,
                    Some(resolved.content),
                )
            }
            None => (Vec::new(), Vec::new(), None),
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

        let post = with_transaction!(self.db_pool, |tx| {
            let post: Post = diesel::update(posts::table)
                .set(PartialPostChangeset {
                    id: update_post.post_id,
                    subject: subject.as_deref(),
                    content: content.as_deref(),
                    content_source: update_post.content.as_deref(),
                    content_lang: content_lang.map(Into::into),
                    link_preview_url: link_preview_url.as_deref(),
                    is_sensitive: update_post.sensitive,
                    updated_at: Timestamp::now_utc(),
                })
                .returning(Post::as_returning())
                .get_result(tx)
                .await?;

            Self::process_mentions(tx, post.account_id, post.id, mentioned_account_ids).await?;
            Self::process_custom_emojis(tx, post.id, custom_emojis).await?;
            Self::process_media_attachments(tx, post.id, &update_post.media_ids).await?;
            NotificationService::notify_on_update_post(tx, post.account_id, post.id).await?;

            Ok::<_, Error>(post)
        })?;

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
            self.search_backend
                .update_in_index(post.clone().into())
                .await?;
        }

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
            .build();

        let existing_repost: Option<Post> = with_connection!(self.db_pool, |db_conn| {
            posts::table
                .filter(
                    posts::reposted_post_id
                        .eq(repost_post.post_id)
                        .and(posts::account_id.eq(repost_post.account_id)),
                )
                .add_post_permission_check(permission_check)
                .select(Post::as_select())
                .first(db_conn)
                .await
                .optional()
        })?;

        if let Some(repost) = existing_repost {
            return Ok(repost);
        }

        let post: Post = with_connection!(self.db_pool, |db_conn| {
            posts::table
                .find(repost_post.post_id)
                .add_post_permission_check(permission_check)
                .select(Post::as_select())
                .get_result(db_conn)
                .await
        })?;

        let id = Uuid::now_v7();
        let url = self.url_service.post_url(id);

        let repost = with_transaction!(self.db_pool, |tx| {
            let new_repost = diesel::insert_into(posts::table)
                .values(NewPost {
                    id,
                    account_id: repost_post.account_id,
                    in_reply_to_id: None,
                    reposted_post_id: Some(post.id),
                    subject: None,
                    content: "",
                    content_source: "",
                    content_lang: post.content_lang,
                    link_preview_url: None,
                    is_sensitive: post.is_sensitive,
                    visibility: repost_post.visibility,
                    is_local: true,
                    url: url.as_str(),
                    created_at: Some(Timestamp::now_utc()),
                })
                .returning(Post::as_returning())
                .get_result(tx)
                .await?;

            NotificationService::notify_on_repost(
                tx,
                post.account_id,
                new_repost.account_id,
                post.id,
            )
            .await?;

            Ok::<_, Error>(new_repost)
        })?;

        self.job_service
            .enqueue(
                Enqueue::builder()
                    .job(DeliverCreate { post_id: repost.id })
                    .build(),
            )
            .await?;

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
            .build();

        let post: Post = with_connection!(self.db_pool, |db_conn| {
            posts::table
                .filter(
                    posts::account_id
                        .eq(unrepost_post.account_id)
                        .and(posts::reposted_post_id.eq(unrepost_post.post_id)),
                )
                .add_post_permission_check(permission_check)
                .select(Post::as_select())
                .first(db_conn)
                .await
        })?;

        self.job_service
            .enqueue(
                Enqueue::builder()
                    .job(DeliverDelete { post_id: post.id })
                    .build(),
            )
            .await?;

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
            .build();

        let post: Post = with_connection!(self.db_pool, |db_conn| {
            posts::table
                .find(post_id)
                .add_post_permission_check(permission_check)
                .select(Post::as_select())
                .get_result(db_conn)
                .await
        })?;

        let id = Uuid::now_v7();
        let url = self.url_service.favourite_url(id);

        let favourite_id = with_transaction!(self.db_pool, |tx| {
            let favourite = diesel::insert_into(posts_favourites::table)
                .values(NewFavourite {
                    id,
                    account_id: favouriting_account_id,
                    post_id: post.id,
                    url,
                    created_at: None,
                })
                .returning(posts_favourites::id)
                .get_result(tx)
                .await?;

            let account_id = accounts::table
                .inner_join(accounts_preferences::table)
                .filter(
                    accounts::id
                        .eq(post.account_id)
                        .and(accounts_preferences::notify_on_favourite.eq(true)),
                )
                .select(accounts::id)
                .get_result::<Uuid>(tx)
                .await
                .optional()?;

            if let Some(account_id) = account_id {
                diesel::insert_into(notifications::table)
                    .values(
                        NewNotification::builder()
                            .receiving_account_id(account_id)
                            .favourite(favouriting_account_id, post.id),
                    )
                    .on_conflict_do_nothing()
                    .execute(tx)
                    .await?;
            }

            Ok::<_, Error>(favourite)
        })?;

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

        let favourite = with_connection!(self.db_pool, |db_conn| {
            Favourite::belonging_to(&post)
                .filter(posts_favourites::account_id.eq(favouriting_account_id))
                .get_result::<Favourite>(db_conn)
                .await
                .optional()
        })?;

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

    /// Get accounts that favourited a post
    ///
    /// Does checks whether the user has access to the post
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, please open an issue.
    pub async fn favourited_by(
        &self,
        get_favourites: GetAccountsInteractingWithPost,
    ) -> Result<impl Stream<Item = Result<Account>> + '_> {
        get_favourites.validate(&LimitContext::default())?;

        let mut query = posts_favourites::table
            .inner_join(accounts::table.on(posts_favourites::account_id.eq(accounts::id)))
            .filter(posts_favourites::post_id.eq(get_favourites.post_id))
            .select(Account::as_select())
            .order(accounts::id.desc())
            .limit(get_favourites.limit as i64)
            .into_boxed();

        if let Some(max_id) = get_favourites.max_id {
            query = query.filter(posts_favourites::id.lt(max_id));
        }
        if let Some(since_id) = get_favourites.since_id {
            query = query.filter(posts_favourites::id.gt(since_id));
        }
        if let Some(min_id) = get_favourites.min_id {
            query = query
                .filter(posts_favourites::id.gt(min_id))
                .order(posts_favourites::id.asc());
        }

        with_connection!(self.db_pool, |db_conn| {
            Ok::<_, Error>(query.load_stream(db_conn).await?.map_err(Error::from))
        })
    }

    /// Get accounts that reblogged a post
    ///
    /// Does checks whether the user has access to the post
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, please open an issue.
    pub async fn reblogged_by(
        &self,
        get_reblogs: GetAccountsInteractingWithPost,
    ) -> Result<impl Stream<Item = Result<Account>> + '_> {
        get_reblogs.validate(&LimitContext::default())?;

        let permission_check = PermissionCheck::builder()
            .fetching_account_id(get_reblogs.fetching_account_id)
            .build();

        let mut query = posts::table
            .add_post_permission_check(permission_check)
            .filter(posts::reposted_post_id.eq(get_reblogs.post_id))
            .into_boxed();

        if let Some(max_id) = get_reblogs.max_id {
            query = query.filter(posts::id.lt(max_id));
        }
        if let Some(since_id) = get_reblogs.since_id {
            query = query.filter(posts::id.gt(since_id));
        }
        if let Some(min_id) = get_reblogs.min_id {
            query = query.filter(posts::id.gt(min_id)).order(posts::id.asc());
        }

        let query = query
            .inner_join(accounts::table.on(accounts::id.eq(posts::account_id)))
            .select(Account::as_select())
            .order(accounts::id.desc())
            .limit(get_reblogs.limit as i64);

        with_connection!(self.db_pool, |db_conn| {
            Ok::<_, Error>(query.load_stream(db_conn).await?.map_err(Error::from))
        })
        .map_err(Error::from)
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
            .build();

        with_connection!(self.db_pool, |db_conn| {
            posts::table
                .find(id)
                .add_post_permission_check(permission_check)
                .select(Post::as_select())
                .get_result(db_conn)
                .await
        })
        .map_err(Error::from)
    }

    /// Get a post's source by its ID
    ///
    /// Does checks whether the user is allowed to fetch the post
    #[allow(clippy::missing_panics_doc)]
    pub async fn get_source_by_id(
        &self,
        id: Uuid,
        fetching_account_id: Option<Uuid>,
    ) -> Result<PostSource> {
        let permission_check = PermissionCheck::builder()
            .fetching_account_id(fetching_account_id)
            .build();

        with_connection!(self.db_pool, |db_conn| {
            posts::table
                .find(id)
                .add_post_permission_check(permission_check)
                .select(PostSource::as_select())
                .get_result(db_conn)
                .await
        })
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
        let load_post = move |in_reply_to_id, permission_check| async move {
            let post = with_connection!(self.db_pool, |db_conn| {
                posts::table
                    .find(in_reply_to_id)
                    .add_post_permission_check(permission_check)
                    .select(Post::as_select())
                    .get_result::<Post>(db_conn)
                    .await
            })?;

            Ok::<_, Error>(post)
        };

        try_stream! {
            let mut last_post = self.get_by_id(id, fetching_account_id).await?;
            let permission_check = PermissionCheck::builder()
                .fetching_account_id(fetching_account_id)
                .build();

            while let Some(in_reply_to_id) = last_post.in_reply_to_id {
                let post = load_post(in_reply_to_id, permission_check).await?;

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
        let load_post = move |id, permission_check| async move {
            let post = with_connection!(self.db_pool, |db_conn| {
                posts::table
                    .filter(posts::in_reply_to_id.eq(id))
                    .add_post_permission_check(permission_check)
                    .select(Post::as_select())
                    .load_stream::<Post>(db_conn)
                    .await
            })?;

            Ok::<_, Error>(post)
        };

        try_stream! {
            let permission_check = PermissionCheck::builder()
                .fetching_account_id(fetching_account_id)
                .build();

            let descendant_stream = load_post(id, permission_check).await?;
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

    async fn get_post_with_access_guard(
        &self,
        post_id: Uuid,
        account_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<Post> {
        let post: Post = with_connection!(self.db_pool, |db_conn| {
            posts::table
                .find(post_id)
                .select(Post::as_select())
                .first(db_conn)
                .await
        })?;

        if post.account_id != account_id {
            if let Some(user_id) = user_id {
                let admin_role_count = with_connection!(self.db_pool, |db_conn| {
                    users_roles::table
                        .filter(
                            users_roles::user_id
                                .eq(user_id)
                                .and(users_roles::role.eq(Role::Administrator)),
                        )
                        .count()
                        .get_result::<i64>(db_conn)
                        .await
                })?;

                if admin_role_count == 0 {
                    bail!(type = ErrorType::Unauthorized, "unauthorised (not an admin)");
                }
            } else {
                bail!(type = ErrorType::Unauthorized, "unauthorised (not logged in)");
            }
        }

        Ok(post)
    }
}

#[cfg(test)]
mod test {
    use crate::post::{CreatePost, PostValidationContext, UpdatePost};
    use garde::Validate;
    use speedy_uuid::Uuid;

    #[test]
    fn new_post_character_limit() {
        let create_post = CreatePost::builder()
            .author_id(Uuid::now_v7())
            .subject(Some("hello".into()))
            .content("world".into())
            .build();

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

        let create_post = CreatePost::builder()
            .author_id(Uuid::now_v7())
            .content(String::new())
            .build();

        assert!(create_post
            .validate(&PostValidationContext {
                character_limit: 25
            })
            .is_err());

        let create_post = CreatePost::builder()
            .author_id(Uuid::now_v7())
            .media_ids(vec![Uuid::now_v7()])
            .content(String::new())
            .build();

        assert!(create_post
            .validate(&PostValidationContext {
                character_limit: 25
            })
            .is_ok());
    }

    #[test]
    fn update_post_character_limit() {
        let update_post = UpdatePost::builder()
            .post_id(Uuid::now_v7())
            .account_id(Uuid::now_v7())
            .subject(Some("hello".into()))
            .content(Some("world".into()))
            .build();

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
