use super::search::{GrpcSearchService, SearchService};
use crate::{
    cache::{Cache, RedisCache},
    config::Configuration,
    error::Result,
    job::{deliver::create::CreateDeliveryContext, Job},
    resolve::PostResolver,
    sanitize::CleanHtmlExt,
};
use chrono::Utc;
use derive_builder::Builder;
use futures_util::FutureExt;
use kitsune_db::{
    custom::{JobState, Visibility},
    entity::{accounts, jobs, posts, posts_mentions, prelude::Posts},
};
use pulldown_cmark::{html, Options, Parser};
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, PaginatorTrait,
    TransactionTrait,
};
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

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct PostService<
    S = GrpcSearchService,
    FPC = RedisCache<str, posts::Model>,
    FUC = RedisCache<str, accounts::Model>,
    WC = RedisCache<str, String>,
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
            self.search_service.add_to_index(post.clone()).await?;
        }

        Ok(post)
    }
}
