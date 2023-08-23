use crate::{job::JobRunnerContext, mapping::IntoActivity, try_join};
use async_trait::async_trait;
use athena::Runnable;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{account::Account, favourite::Favourite, user::User},
    schema::{accounts, posts, posts_favourites, users},
};
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverFavourite {
    pub favourite_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverFavourite {
    type Context = JobRunnerContext;
    type Error = eyre::Report;

    #[instrument(skip_all, fields(favourite_id = %self.favourite_id))]
    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let (favourite, ((account, user), inbox_url)) = ctx
            .state
            .db_pool
            .with_connection(|mut db_conn| async move {
                let favourite = posts_favourites::table
                    .find(self.favourite_id)
                    .get_result::<Favourite>(&mut db_conn)
                    .await?;

                let account_user_fut = accounts::table
                    .find(favourite.account_id)
                    .inner_join(users::table)
                    .select(<(Account, User)>::as_select())
                    .get_result(&mut db_conn);

                let inbox_url_fut = posts::table
                    .find(favourite.post_id)
                    .inner_join(accounts::table)
                    .select(accounts::inbox_url)
                    .get_result::<Option<String>>(&mut db_conn);

                Ok::<_, Self::Error>((favourite, try_join!(account_user_fut, inbox_url_fut)?))
            })
            .await?;

        if let Some(ref inbox_url) = inbox_url {
            let activity = favourite.into_activity(&ctx.state).await?;

            ctx.deliverer
                .deliver(inbox_url, &account, &user, &activity)
                .await?;
        }

        Ok(())
    }
}
