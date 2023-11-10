use crate::{
    error::{ApiError, Error, Result},
    sanitize::CleanHtmlExt,
    util::timestamp_to_uuid,
};
use diesel::{ExpressionMethods, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures_util::FutureExt;
use http::Uri;
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    model::{
        account::Account,
        media_attachment::{NewMediaAttachment, NewPostMediaAttachment},
        mention::NewMention,
        post::{FullPostChangeset, NewPost, Post, PostConflictChangeset, Visibility},
    },
    schema::{media_attachments, posts, posts_media_attachments, posts_mentions},
    PgPool,
};
use kitsune_embed::Client as EmbedClient;
use kitsune_language::{DetectionBackend, Language};
use kitsune_search::{AnySearchBackend, SearchBackend};
use kitsune_type::ap::{object::MediaAttachment, Object, Tag, TagType};
use kitsune_util::CowBox;
use pulldown_cmark::{html, Options, Parser};
use scoped_futures::ScopedFutureExt;
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

pub mod deliverer;
pub mod fetcher;

pub use self::{deliverer::Deliverer, fetcher::Fetcher};

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
                        account_id: author.id,
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
    fetcher: &'a Fetcher,
    search_backend: &'a AnySearchBackend,
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
    }: ProcessNewObject<'_>,
) -> Result<PreprocessedObject<'_>> {
    let attributed_to = object.attributed_to().ok_or(ApiError::BadRequest)?;
    let user = if let Some(author) = author {
        CowBox::borrowed(author)
    } else {
        if Uri::try_from(attributed_to)?.authority() != Uri::try_from(&object.id)?.authority() {
            return Err(ApiError::BadRequest.into());
        }

        CowBox::owned(fetcher.fetch_actor(attributed_to.into()).await?)
    };

    let visibility = Visibility::from_activitypub(&user, &object).unwrap();
    let in_reply_to_id = if let Some(ref in_reply_to) = object.in_reply_to {
        fetcher
            .fetch_object_inner(in_reply_to, call_depth + 1)
            .await?
            .map(|post| post.id)
    } else {
        None
    };

    if object.media_type.as_deref() == Some("text/markdown") {
        let parser = Parser::new_ext(&object.content, Options::all());
        let mut buf = String::new();
        html::push_html(&mut buf, parser);
        object.content = buf;
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
        kitsune_language::detect_language(DetectionBackend::default(), object.content.as_str());

    Ok(PreprocessedObject {
        user,
        visibility,
        in_reply_to_id,
        link_preview_url,
        content_lang,
        db_pool,
        object,
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
        search_backend,
    } = preprocess_object(process_data).boxed().await?;

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

                Ok::<_, Error>(new_post)
            }
            .scope_boxed()
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
            .scope_boxed()
        })
        .await?;

    if post.visibility == Visibility::Public || post.visibility == Visibility::Unlisted {
        search_backend.update_in_index(post.clone().into()).await?;
    }

    Ok(post)
}
