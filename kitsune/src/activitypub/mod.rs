use crate::{
    error::{ApiError, Error, Result},
    sanitize::CleanHtmlExt,
};
use diesel::SelectableHelper;
use diesel_async::{
    scoped_futures::ScopedFutureExt, AsyncConnection, AsyncPgConnection, RunQueryDsl,
};
use kitsune_db::{
    model::{
        account::Account,
        media_attachment::{NewMediaAttachment, NewPostMediaAttachment},
        mention::NewMention,
        post::{NewPost, Post, PostConflictChangeset, Visibility},
    },
    schema::{media_attachments, posts, posts_media_attachments, posts_mentions},
};
use kitsune_search::{SearchBackend, SearchService};
use kitsune_type::ap::{object::MediaAttachment, Object, Tag, TagType};
use pulldown_cmark::{html, Options, Parser};
use typed_builder::TypedBuilder;
use uuid::{Timestamp, Uuid};

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
    #[builder(default, setter(strip_option))]
    author: Option<Account>,
    #[builder(default = 0)]
    call_depth: u32,
    db_conn: &'a mut AsyncPgConnection,
    object: Object,
    fetcher: &'a Fetcher,
    search_service: &'a SearchService,
}

#[allow(clippy::missing_panics_doc)]
pub async fn process_new_object(
    ProcessNewObject {
        author,
        call_depth,
        db_conn,
        mut object,
        fetcher,
        search_service,
    }: ProcessNewObject<'_>,
) -> Result<Post> {
    let attributed_to = object.attributed_to().ok_or(ApiError::BadRequest)?;
    let user = if let Some(author) = author {
        author
    } else {
        fetcher.fetch_actor(attributed_to.into()).await?
    };

    let visibility = Visibility::from_activitypub(&user, &object).unwrap();

    #[allow(clippy::cast_sign_loss)]
    let uuid_timestamp = Timestamp::from_unix(
        uuid::NoContext,
        object.published.unix_timestamp() as u64,
        object.published.nanosecond(),
    );

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

    if let Some(ref name) = object.name {
        object.content = format!(
            r#"<p><a href="{}">{}</a></p>{}"#,
            object.id, name, object.content
        );
    }
    object.clean_html();

    let post = db_conn
        .transaction(|tx| {
            async move {
                let new_post = diesel::insert_into(posts::table)
                    .values(NewPost {
                        id: Uuid::new_v7(uuid_timestamp),
                        account_id: user.id,
                        in_reply_to_id,
                        reposted_post_id: None,
                        subject: object.summary.as_deref(),
                        content: object.content.as_str(),
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
        search_service.add_to_index(post.clone().into()).await?;
    }

    Ok(post)
}
