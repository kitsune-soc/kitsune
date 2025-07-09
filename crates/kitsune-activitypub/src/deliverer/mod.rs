use crate::{
    InboxResolver,
    mapping::{self, IntoActivity},
};
use async_trait::async_trait;
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl,
    SelectableHelper,
};
use diesel_async::RunQueryDsl;
use futures_util::TryStreamExt;
use iso8601_timestamp::Timestamp;
use kitsune_core::traits::{Deliverer as DelivererTrait, deliverer::Action};
use kitsune_db::{
    PgPool,
    model::{account::Account, favourite::Favourite, follower::Follow, post::Post, user::User},
    schema::{accounts, posts, users},
    with_connection,
};
use kitsune_error::Result;
use kitsune_service::attachment::AttachmentService;
use kitsune_type::ap::{Activity, ActivityType, ObjectField, ap_context};
use kitsune_url::UrlService;
use kitsune_util::try_join;
use triomphe::Arc;
use typed_builder::TypedBuilder;

pub mod core;

const MAX_CONCURRENT_REQUESTS: usize = 10;

#[derive(TypedBuilder)]
pub struct Service {
    attachment: AttachmentService,
    url: UrlService,
}

#[derive(TypedBuilder)]
#[builder(build_method(into = Arc<Deliverer>))]
pub struct Deliverer {
    core: core::Deliverer,
    db_pool: PgPool,
    inbox_resolver: InboxResolver,
    service: Service,
}

impl Deliverer {
    fn mapping_state(&self) -> mapping::State<'_> {
        let service = mapping::Service::builder()
            .attachment(&self.service.attachment)
            .url(&self.service.url)
            .build();

        mapping::State::builder()
            .db_pool(&self.db_pool)
            .service(service)
            .build()
    }

    async fn accept_follow(&self, follow: Follow) -> Result<()> {
        let (follower_inbox_url, (followed_account, followed_user)): (String, _) =
            with_connection!(self.db_pool, |db_conn| {
                let follower_inbox_url_fut = accounts::table
                    .find(follow.follower_id)
                    .select(accounts::inbox_url.assume_not_null())
                    .get_result::<String>(db_conn);

                let followed_info_fut = accounts::table
                    .find(follow.account_id)
                    .inner_join(users::table.on(accounts::id.eq(users::account_id)))
                    .select(<(Account, User)>::as_select())
                    .get_result::<(Account, User)>(db_conn);

                try_join!(follower_inbox_url_fut, followed_info_fut)
            })?;

        let followed_account_url = self.service.url.user_url(followed_account.id);

        // Constructing this here is against our idea of the `IntoActivity` and `IntoObject` traits
        // But I'm not sure how I could encode these into the form of these two traits
        // So we make an exception for this
        //
        // If someone has a better idea, please open an issue
        let accept_activity = Activity {
            context: ap_context(),
            id: format!("{}#accept", follow.url),
            r#type: ActivityType::Accept,
            actor: followed_account_url,
            object: ObjectField::Url(follow.url),
            published: Timestamp::now_utc(),
        };

        self.core
            .deliver(
                &follower_inbox_url,
                &followed_account,
                &followed_user,
                &accept_activity,
            )
            .await?;

        Ok(())
    }

    async fn create_or_repost(&self, post: Post) -> Result<()> {
        let (account, user) = with_connection!(self.db_pool, |db_conn| {
            accounts::table
                .find(post.account_id)
                .inner_join(users::table)
                .select(<(Account, User)>::as_select())
                .get_result::<(Account, User)>(db_conn)
                .await
        })?;

        let inbox_stream = self
            .inbox_resolver
            .resolve(&post)
            .await?
            .try_chunks(MAX_CONCURRENT_REQUESTS)
            .map_err(|err| err.1);

        let activity = post.into_activity(self.mapping_state()).await?;

        // TODO: Should we deliver to the inboxes that are contained inside a `TryChunksError`?
        self.core
            .deliver_many(&account, &user, &activity, inbox_stream)
            .await?;

        Ok(())
    }

    async fn delete_or_unrepost(&self, post: Post) -> Result<()> {
        let account_user_data = with_connection!(self.db_pool, |db_conn| {
            accounts::table
                .find(post.account_id)
                .inner_join(users::table)
                .select(<(Account, User)>::as_select())
                .get_result::<(Account, User)>(db_conn)
                .await
                .optional()
        })?;

        let Some((account, user)) = account_user_data else {
            return Ok(());
        };

        let inbox_stream = self
            .inbox_resolver
            .resolve(&post)
            .await?
            .try_chunks(MAX_CONCURRENT_REQUESTS)
            .map_err(|err| err.1);

        let delete_activity = post.into_negate_activity(self.mapping_state()).await?;

        // TODO: Should we deliver to the inboxes that are contained inside a `TryChunksError`?
        self.core
            .deliver_many(&account, &user, &delete_activity, inbox_stream)
            .await?;

        Ok(())
    }

    async fn favourite(&self, favourite: Favourite) -> Result<()> {
        let ((account, user), inbox_url) = with_connection!(self.db_pool, |db_conn| {
            let account_user_fut = accounts::table
                .find(favourite.account_id)
                .inner_join(users::table)
                .select(<(Account, User)>::as_select())
                .get_result(db_conn);

            let inbox_url_fut = posts::table
                .find(favourite.post_id)
                .inner_join(accounts::table)
                .select(accounts::inbox_url)
                .get_result::<Option<String>>(db_conn);

            try_join!(account_user_fut, inbox_url_fut)
        })?;

        if let Some(ref inbox_url) = inbox_url {
            let activity = favourite.into_activity(self.mapping_state()).await?;

            self.core
                .deliver(inbox_url, &account, &user, &activity)
                .await?;
        }

        Ok(())
    }

    async fn follow(&self, follow: Follow) -> Result<()> {
        let ((follower, follower_user), followed_inbox) =
            with_connection!(self.db_pool, |db_conn| {
                let follower_info_fut = accounts::table
                    .find(follow.follower_id)
                    .inner_join(users::table)
                    .select(<(Account, User)>::as_select())
                    .get_result::<(Account, User)>(db_conn);

                let followed_inbox_fut = accounts::table
                    .find(follow.account_id)
                    .select(accounts::inbox_url)
                    .get_result::<Option<String>>(db_conn);

                try_join!(follower_info_fut, followed_inbox_fut)
            })?;

        if let Some(followed_inbox) = followed_inbox {
            let follow_activity = follow.into_activity(self.mapping_state()).await?;

            self.core
                .deliver(&followed_inbox, &follower, &follower_user, &follow_activity)
                .await?;
        }

        Ok(())
    }

    async fn reject_follow(&self, follow: Follow) -> Result<()> {
        let (follower_inbox_url, (followed_account, followed_user), _delete_result) =
            with_connection!(self.db_pool, |db_conn| {
                let follower_inbox_url_fut = accounts::table
                    .find(follow.follower_id)
                    .select(accounts::inbox_url.assume_not_null())
                    .get_result::<String>(db_conn);

                let followed_info_fut = accounts::table
                    .find(follow.account_id)
                    .inner_join(users::table.on(accounts::id.eq(users::account_id)))
                    .select(<(Account, User)>::as_select())
                    .get_result::<(Account, User)>(db_conn);

                let delete_fut = diesel::delete(&follow).execute(db_conn);

                try_join!(follower_inbox_url_fut, followed_info_fut, delete_fut)
            })?;

        let followed_account_url = self.service.url.user_url(followed_account.id);

        // Constructing this here is against our idea of the `IntoActivity` and `IntoObject` traits
        // But I'm not sure how I could encode these into the form of these two traits
        // So we make an exception for this
        //
        // If someone has a better idea, please open an issue
        let reject_activity = Activity {
            context: ap_context(),
            id: format!("{}#reject", follow.url),
            r#type: ActivityType::Reject,
            actor: followed_account_url,
            object: ObjectField::Url(follow.url),
            published: Timestamp::now_utc(),
        };

        self.core
            .deliver(
                &follower_inbox_url,
                &followed_account,
                &followed_user,
                &reject_activity,
            )
            .await?;

        Ok(())
    }

    async fn unfavourite(&self, favourite: Favourite) -> Result<()> {
        let ((account, user), inbox_url) = with_connection!(self.db_pool, |db_conn| {
            let account_user_fut = accounts::table
                .find(favourite.account_id)
                .inner_join(users::table)
                .select(<(Account, User)>::as_select())
                .get_result(db_conn);

            let inbox_url_fut = posts::table
                .find(favourite.post_id)
                .inner_join(accounts::table)
                .select(accounts::inbox_url)
                .get_result::<Option<String>>(db_conn);

            try_join!(account_user_fut, inbox_url_fut)
        })?;

        if let Some(ref inbox_url) = inbox_url {
            let activity = favourite.into_negate_activity(self.mapping_state()).await?;
            self.core
                .deliver(inbox_url, &account, &user, &activity)
                .await?;
        }

        Ok(())
    }

    async fn unfollow(&self, follow: Follow) -> Result<()> {
        let ((follower, follower_user), followed_account_inbox_url) =
            with_connection!(self.db_pool, |db_conn| {
                let follower_info_fut = accounts::table
                    .find(follow.follower_id)
                    .inner_join(users::table)
                    .select(<(Account, User)>::as_select())
                    .get_result::<(Account, User)>(db_conn);

                let followed_account_inbox_url_fut = accounts::table
                    .find(follow.account_id)
                    .select(accounts::inbox_url)
                    .get_result::<Option<String>>(db_conn);

                try_join!(follower_info_fut, followed_account_inbox_url_fut)
            })?;

        if let Some(ref followed_account_inbox_url) = followed_account_inbox_url {
            let follow_activity = follow.into_negate_activity(self.mapping_state()).await?;

            self.core
                .deliver(
                    followed_account_inbox_url,
                    &follower,
                    &follower_user,
                    &follow_activity,
                )
                .await?;
        }

        Ok(())
    }

    async fn update_account(&self, account: Account) -> Result<()> {
        let user = with_connection!(self.db_pool, |db_conn| {
            users::table
                .filter(users::account_id.eq(account.id))
                .select(User::as_select())
                .get_result(db_conn)
                .await
                .optional()
        })?;

        let Some(user) = user else {
            return Ok(());
        };

        let activity = account.clone().into_activity(self.mapping_state()).await?;
        let inbox_stream = self
            .inbox_resolver
            .resolve_followers(&account)
            .await?
            .try_chunks(MAX_CONCURRENT_REQUESTS)
            .map_err(|err| err.1);

        self.core
            .deliver_many(&account, &user, &activity, inbox_stream)
            .await?;

        Ok(())
    }

    async fn update_post(&self, post: Post) -> Result<()> {
        let post_account_user_data = with_connection!(self.db_pool, |db_conn| {
            posts::table
                .find(post.id)
                .inner_join(accounts::table)
                .inner_join(users::table.on(accounts::id.eq(users::account_id)))
                .select(<(Account, User)>::as_select())
                .get_result(db_conn)
                .await
                .optional()
        })?;

        let Some((account, user)) = post_account_user_data else {
            return Ok(());
        };

        let inbox_stream = self
            .inbox_resolver
            .resolve(&post)
            .await?
            .try_chunks(MAX_CONCURRENT_REQUESTS)
            .map_err(|err| err.1);

        let mut activity = post.into_activity(self.mapping_state()).await?;

        // Patch in the update
        activity.r#type = ActivityType::Update;

        self.core
            .deliver_many(&account, &user, &activity, inbox_stream)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl DelivererTrait for Deliverer {
    async fn deliver(&self, action: Action) -> Result<()> {
        match action {
            Action::AcceptFollow(follow) => self.accept_follow(follow).await,
            Action::Create(post) | Action::Repost(post) => self.create_or_repost(post).await,
            Action::Delete(post) | Action::Unrepost(post) => self.delete_or_unrepost(post).await,
            Action::Favourite(favourite) => self.favourite(favourite).await,
            Action::Follow(follow) => self.follow(follow).await,
            Action::RejectFollow(follow) => self.reject_follow(follow).await,
            Action::Unfavourite(favourite) => self.unfavourite(favourite).await,
            Action::Unfollow(follow) => self.unfollow(follow).await,
            Action::UpdateAccount(account) => self.update_account(account).await,
            Action::UpdatePost(post) => self.update_post(post).await,
        }
    }
}
