use diesel::SelectableHelper;
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncPgConnection, RunQueryDsl};
use kitsune_db::{
    model::{
        account::{Account, ActorType, NewAccount},
        user::{NewUser, User},
    },
    schema::{accounts, users},
};
use kitsune_test::database_test;
use speedy_uuid::Uuid;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

async fn create_account(conn: &mut AsyncPgConnection, username: &str) -> Result<Account> {
    diesel::insert_into(accounts::table)
        .values(NewAccount {
            id: Uuid::now_v7(),
            actor_type: ActorType::Person,
            display_name: None,
            note: None,
            username,
            locked: false,
            local: true,
            domain: "kitsune.example",
            url: format!("https://kitsune.example/users/{username}").as_str(),
            featured_collection_url: None,
            followers_url: None,
            following_url: None,
            inbox_url: None,
            outbox_url: None,
            shared_inbox_url: None,
            public_key: "---WHATEVER---",
            public_key_id: format!("can we abandon rsa already? ({username}'s key)").as_str(),
            created_at: None,
        })
        .returning(Account::as_returning())
        .get_result(conn)
        .await
        .map_err(Into::into)
}

async fn create_user(conn: &mut AsyncPgConnection, username: &str) -> Result<User> {
    let account = create_account(conn, Uuid::now_v7().to_string().as_str()).await?;

    diesel::insert_into(users::table)
        .values(NewUser {
            id: Uuid::now_v7(),
            account_id: account.id,
            oidc_id: None,
            username,
            email: format!("{username}@kitsune.example").as_str(),
            password: None,
            domain: "kitsune.example",
            private_key: "---WHATEVER---",
            confirmation_token: Uuid::now_v7().to_string().as_str(),
        })
        .returning(User::as_returning())
        .get_result(conn)
        .await
        .map_err(Into::into)
}

#[tokio::test]
async fn accounts_username() {
    database_test(|db_pool| async move {
        db_pool
            .with_connection(|conn| {
                async move {
                    let initial_insert = create_account(conn, "aumetra").await;
                    assert!(initial_insert.is_ok());

                    let case_mutation = create_account(conn, "AuMeTrA").await;
                    assert!(case_mutation.is_err());

                    let unicode_mutation_1 = create_account(conn, "äumeträ").await;
                    assert!(unicode_mutation_1.is_err());

                    let unicode_mutation_2 = create_account(conn, "🅰umetr🅰").await;
                    assert!(unicode_mutation_2.is_err());

                    let unicode_case_mutation = create_account(conn, "🅰UMETR🅰").await;
                    assert!(unicode_case_mutation.is_err());

                    Result::Ok(())
                }
                .scoped()
            })
            .await
            .unwrap();
    })
    .await;
}

#[tokio::test]
async fn users_username() {
    database_test(|db_pool| async move {
        db_pool
            .with_connection(|conn| {
                async move {
                    let initial_insert = create_user(conn, "aumetra").await;
                    assert!(initial_insert.is_ok());

                    let case_mutation = create_user(conn, "AuMeTrA").await;
                    assert!(case_mutation.is_err());

                    let unicode_mutation_1 = create_user(conn, "äumeträ").await;
                    assert!(unicode_mutation_1.is_err());

                    let unicode_mutation_2 = create_user(conn, "🅰umetr🅰").await;
                    assert!(unicode_mutation_2.is_err());

                    let unicode_case_mutation = create_user(conn, "🅰UMETR🅰").await;
                    assert!(unicode_case_mutation.is_err());

                    Result::Ok(())
                }
                .scoped()
            })
            .await
            .unwrap();
    })
    .await;
}
