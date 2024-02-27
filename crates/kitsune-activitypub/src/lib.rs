#[macro_use]
extern crate tracing;

use crate::error::{Error, Result};
use diesel::{ExpressionMethods, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures_util::{stream, StreamExt, TryStreamExt};
use http::Uri;
use iso8601_timestamp::Timestamp;
use kitsune_config::language_detection::Configuration as LanguageDetectionConfig;
use kitsune_core::traits::{fetcher::PostFetchOptions, Fetcher as FetcherTrait};
use kitsune_db::{
    model::{
        account::Account,
        custom_emoji::PostCustomEmoji,
        media_attachment::{NewMediaAttachment, NewPostMediaAttachment},
        mention::NewMention,
        post::{FullPostChangeset, NewPost, Post, PostConflictChangeset, Visibility},
    },
    schema::{
        media_attachments, posts, posts_custom_emojis, posts_media_attachments, posts_mentions,
    },
    PgPool,
};
use kitsune_embed::Client as EmbedClient;
use kitsune_language::Language;
use kitsune_search::{AnySearchBackend, SearchBackend};
use kitsune_type::ap::{object::MediaAttachment, Object, Tag, TagType};
use kitsune_util::{convert::timestamp_to_uuid, process, sanitize::CleanHtmlExt, CowBox};
use scoped_futures::ScopedFutureExt;
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

pub mod deliverer;
pub mod error;
pub mod fetcher;
pub mod inbox_resolver;
pub mod mapping;

pub use self::{
    deliverer::{core::Deliverer as CoreDeliverer, Deliverer},
    fetcher::Fetcher,
    inbox_resolver::InboxResolver,
};

async fn handle_mentions(
    db_conn: &mut AsyncPgConnection,
    author: &Account,
    post_id: Uuid,
    mentions: &[Tag],
) -> Result<()> {
    let mention_iter = mentions
        .iter()
        .filter(|mention| mention.r#type == TagType::Mention);

    if mention_iter.clone().count() == 0 {
        return Ok(());
    }

    diesel::insert_into(posts_mentions::table)
        .values(
            mention_iter
                .map(|mention| NewMention {
                    post_id,
                    account_id: author.id,
                    mention_text: mention.name.as_str(),
                })
                .collect::<Vec<NewMention<'_>>>(),
        )
        .on_conflict_do_nothing()
        .execute(db_conn)
        .await?;

    Ok(())
}

async fn handle_custom_emojis(
    db_conn: &mut AsyncPgConnection,
    post_id: Uuid,
    fetcher: &dyn FetcherTrait,
    tags: &[Tag],
) -> Result<()> {
    let emoji_iter = tags.iter().filter(|tag| tag.r#type == TagType::Emoji);

    let emoji_count = emoji_iter.clone().count();
    if emoji_count == 0 {
        return Ok(());
    }

    let futures = stream::iter(emoji_iter).filter_map(|emoji| async move {
        let remote_id = emoji.id.as_ref()?;
        let emoji = fetcher
            .fetch_emoji(remote_id)
            .await
            .transpose()?
            .map(move |f| (f, emoji));

        Some(emoji)
    });

    let emojis = futures
        .map_ok(|(resolved_emoji, emoji_tag)| PostCustomEmoji {
            post_id,
            custom_emoji_id: resolved_emoji.id,
            emoji_text: emoji_tag.name.clone(),
        })
        .try_collect::<Vec<PostCustomEmoji>>()
        .await
        .map_err(Error::FetchEmoji)?;

    diesel::insert_into(posts_custom_emojis::table)
        .values(emojis)
        .on_conflict_do_nothing()
        .execute(db_conn)
        .await?;

    Ok(())
}

/// Process a bunch of ActivityPub attachments
///
/// # Returns
///
/// Returns a vector containing the IDs of the newly contained media attachments
pub async fn process_attachments(
    db_conn: &mut AsyncPgConnection,
    author: &Account,
    attachments: &[MediaAttachment],
) -> Result<Vec<Uuid>> {
    if attachments.is_empty() {
        return Ok(Vec::new());
    }
    let attachment_ids: Vec<Uuid> = (0..attachments.len()).map(|_| Uuid::now_v7()).collect();

    diesel::insert_into(media_attachments::table)
        .values(
            attachments
                .iter()
                .zip(attachment_ids.iter().copied())
                .filter_map(|(attachment, attachment_id)| {
                    let content_type = attachment
                        .media_type
                        .as_deref()
                        .or_else(|| mime_guess::from_path(&attachment.url).first_raw())?;

                    Some(NewMediaAttachment {
                        id: attachment_id,
                        account_id: Some(author.id),
                        content_type,
                        description: attachment.name.as_deref(),
                        blurhash: attachment.blurhash.as_deref(),
                        file_path: None,
                        remote_url: Some(attachment.url.as_str()),
                    })
                })
                .collect::<Vec<NewMediaAttachment<'_>>>(),
        )
        .returning(media_attachments::id)
        .load(db_conn)
        .await
        .map_err(Error::from)
}

#[derive(TypedBuilder)]
pub struct ProcessNewObject<'a> {
    #[builder(default, setter(into, strip_option))]
    author: Option<&'a Account>,
    #[builder(default = 0)]
    call_depth: u32,
    db_pool: &'a PgPool,
    embed_client: Option<&'a EmbedClient>,
    object: Box<Object>,
    fetcher: &'a dyn FetcherTrait,
    search_backend: &'a AnySearchBackend,
    language_detection_config: LanguageDetectionConfig,
}

#[derive(TypedBuilder)]
struct PreprocessedObject<'a> {
    user: CowBox<'a, Account>,
    visibility: Visibility,
    in_reply_to_id: Option<Uuid>,
    link_preview_url: Option<String>,
    content_lang: Language,
    db_pool: &'a PgPool,
    object: Box<Object>,
    fetcher: &'a dyn FetcherTrait,
    search_backend: &'a AnySearchBackend,
}

#[allow(clippy::missing_panics_doc)]
async fn preprocess_object(
    ProcessNewObject {
        author,
        call_depth,
        db_pool,
        embed_client,
        mut object,
        fetcher,
        search_backend,
        language_detection_config,
    }: ProcessNewObject<'_>,
) -> Result<PreprocessedObject<'_>> {
    let user = if let Some(author) = author {
        CowBox::borrowed(author)
    } else {
        if Uri::try_from(&object.attributed_to)?.authority()
            != Uri::try_from(&object.id)?.authority()
        {
            return Err(Error::InvalidDocument);
        }

        let Some(author) = fetcher
            .fetch_account(object.attributed_to.as_str().into())
            .await
            .map_err(Error::FetchAccount)?
        else {
            return Err(Error::NotFound);
        };

        CowBox::boxed(author)
    };

    let visibility = Visibility::from_activitypub(&user, &object).unwrap();
    let in_reply_to_id = if let Some(ref in_reply_to) = object.in_reply_to {
        fetcher
            .fetch_post(
                PostFetchOptions::builder()
                    .url(in_reply_to)
                    .call_depth(call_depth + 1)
                    .build(),
            )
            .await
            .map_err(Error::FetchPost)?
            .map(|post| post.id)
    } else {
        None
    };

    if object.media_type.as_deref() == Some("text/markdown") {
        object.content = process::markdown(&object.content);
    }

    let link_preview_url = if let Some(embed_client) = embed_client {
        embed_client
            .fetch_embed_for_fragment(&object.content)
            .await?
            .map(|fragment_embed| fragment_embed.url)
    } else {
        None
    };

    if let Some(ref name) = object.name {
        object.content = format!(
            r#"<p><a href="{}">{}</a></p>{}"#,
            object.id, name, object.content
        );
    }
    object.clean_html();

    let content_lang =
        kitsune_language::detect_language(language_detection_config, object.content.as_str());

    Ok(PreprocessedObject {
        user,
        visibility,
        in_reply_to_id,
        link_preview_url,
        content_lang,
        db_pool,
        object,
        fetcher,
        search_backend,
    })
}

#[allow(clippy::missing_panics_doc)]
pub async fn process_new_object(process_data: ProcessNewObject<'_>) -> Result<Post> {
    let PreprocessedObject {
        user,
        visibility,
        in_reply_to_id,
        link_preview_url,
        content_lang,
        db_pool,
        object,
        fetcher,
        search_backend,
    } = preprocess_object(process_data).await?;

    let post = db_pool
        .with_transaction(|tx| {
            async move {
                let new_post = diesel::insert_into(posts::table)
                    .values(NewPost {
                        id: timestamp_to_uuid(object.published),
                        account_id: user.id,
                        in_reply_to_id,
                        reposted_post_id: None,
                        subject: object.summary.as_deref(),
                        content: object.content.as_str(),
                        content_source: "",
                        content_lang: content_lang.into(),
                        link_preview_url: link_preview_url.as_deref(),
                        is_sensitive: object.sensitive,
                        visibility,
                        is_local: false,
                        url: object.id.as_str(),
                        created_at: Some(object.published),
                    })
                    .on_conflict(posts::url)
                    .do_update()
                    .set(PostConflictChangeset {
                        subject: object.summary.as_deref(),
                        content: object.content.as_str(),
                    })
                    .returning(Post::as_returning())
                    .get_result::<Post>(tx)
                    .await?;

                let attachment_ids = process_attachments(tx, &user, &object.attachment).await?;
                diesel::insert_into(posts_media_attachments::table)
                    .values(
                        attachment_ids
                            .into_iter()
                            .map(|attachment_id| NewPostMediaAttachment {
                                post_id: new_post.id,
                                media_attachment_id: attachment_id,
                            })
                            .collect::<Vec<NewPostMediaAttachment>>(),
                    )
                    .execute(tx)
                    .await?;

                handle_mentions(tx, &user, new_post.id, &object.tag).await?;
                handle_custom_emojis(tx, new_post.id, fetcher, &object.tag).await?;

                Ok::<_, Error>(new_post)
            }
            .scoped()
        })
        .await?;

    if post.visibility == Visibility::Public || post.visibility == Visibility::Unlisted {
        search_backend.add_to_index(post.clone().into()).await?;
    }

    Ok(post)
}

#[allow(clippy::missing_panics_doc)]
pub async fn update_object(process_data: ProcessNewObject<'_>) -> Result<Post> {
    let PreprocessedObject {
        user,
        visibility,
        in_reply_to_id,
        link_preview_url,
        content_lang,
        db_pool,
        object,
        fetcher: _,
        search_backend,
    } = preprocess_object(process_data).await?;

    let post = db_pool
        .with_transaction(|tx| {
            async move {
                let updated_post = diesel::update(posts::table)
                    .filter(posts::url.eq(object.id.as_str()))
                    .set(FullPostChangeset {
                        account_id: user.id,
                        in_reply_to_id,
                        reposted_post_id: None,
                        subject: object.summary.as_deref(),
                        content: object.content.as_str(),
                        content_source: "",
                        content_lang: content_lang.into(),
                        link_preview_url: link_preview_url.as_deref(),
                        is_sensitive: object.sensitive,
                        visibility,
                        is_local: false,
                        updated_at: Timestamp::now_utc(),
                    })
                    .returning(Post::as_returning())
                    .get_result::<Post>(tx)
                    .await?;

                let attachment_ids = process_attachments(tx, &user, &object.attachment).await?;
                diesel::insert_into(posts_media_attachments::table)
                    .values(
                        attachment_ids
                            .into_iter()
                            .map(|attachment_id| NewPostMediaAttachment {
                                post_id: updated_post.id,
                                media_attachment_id: attachment_id,
                            })
                            .collect::<Vec<NewPostMediaAttachment>>(),
                    )
                    .on_conflict_do_nothing()
                    .execute(tx)
                    .await?;

                handle_mentions(tx, &user, updated_post.id, &object.tag).await?;

                Ok::<_, Error>(updated_post)
            }
            .scoped()
        })
        .await?;

    if post.visibility == Visibility::Public || post.visibility == Visibility::Unlisted {
        search_backend.update_in_index(post.clone().into()).await?;
    }

    Ok(post)
}
