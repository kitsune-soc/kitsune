use crate::{
    error::{ApiError, Error, Result},
    sanitize::CleanHtmlExt,
    service::search::{SearchBackend, SearchService},
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
use kitsune_type::ap::{object::MediaAttachment, Object, Tag, TagType};
use typed_builder::TypedBuilder;
use uuid::{Timestamp, Uuid};

pub mod deliverer;
pub mod fetcher;

pub use self::{deliverer::Deliverer, fetcher::Fetcher};

async fn handle_attachments(
    db_conn: &mut AsyncPgConnection,
    author: &Account,
    post_id: Uuid,
    attachments: Vec<MediaAttachment>,
) -> Result<()> {
    if attachments.is_empty() {
        return Ok(());
    }
    let attachment_ids: Vec<Uuid> = (0..attachments.len()).map(|_| Uuid::now_v7()).collect();

    diesel::insert_into(media_attachments::table)
        .values(
            attachments
                .iter()
                .zip(attachment_ids.iter().copied())
                .map(|(attachment, attachment_id)| NewMediaAttachment {
                    id: attachment_id,
                    account_id: author.id,
                    content_type: attachment.media_type.as_str(),
                    description: attachment.name.as_deref(),
                    blurhash: attachment.blurhash.as_deref(),
                    file_path: None,
                    remote_url: Some(attachment.url.as_str()),
                })
                .collect::<Vec<NewMediaAttachment<'_>>>(),
        )
        .execute(db_conn)
        .await?;

    diesel::insert_into(posts_media_attachments::table)
        .values(
            attachment_ids
                .into_iter()
                .map(|attachment_id| NewPostMediaAttachment {
                    post_id,
                    media_attachment_id: attachment_id,
                })
                .collect::<Vec<NewPostMediaAttachment>>(),
        )
        .execute(db_conn)
        .await?;

    Ok(())
}

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
        .execute(db_conn)
        .await?;

    Ok(())
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
    object.clean_html();

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

    let in_reply_to_id = if let Some(in_reply_to) = object.in_reply_to {
        fetcher
            .fetch_object_inner(&in_reply_to, call_depth + 1)
            .await?
            .map(|post| post.id)
    } else {
        None
    };

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

                handle_attachments(tx, &user, new_post.id, object.attachment).await?;
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
