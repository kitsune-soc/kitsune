use diesel::SelectableHelper;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    insert::{NewAccount, NewUser},
    model::{Account, Domain, User},
    schema::{accounts, domains, users},
    types::{AccountType, Protocol},
    with_connection_panicky,
};
use kitsune_test::database_test;
use speedy_uuid::Uuid;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

async fn create_account(conn: &mut AsyncPgConnection, username: &str) -> Result<Account> {
    diesel::insert_into(domains::table)
        .values(&Domain {
            domain: "kitsune.example".into(),
            owner_id: None,
            challenge_value: None,
            globally_available: false,
            verified_at: Some(Timestamp::now_utc()),
            created_at: Timestamp::now_utc(),
            updated_at: Timestamp::now_utc(),
        })
        .on_conflict_do_nothing()
        .execute(conn)
        .await?;

    diesel::insert_into(accounts::table)
        .values(NewAccount {
            id: Uuid::now_v7(),
            account_type: AccountType::Person,
            protocol: Protocol::supported(),
            avatar_id: None,
            header_id: None,
            display_name: None,
            note: None,
            username,
            locked: false,
            local: true,
            domain: "kitsune.example",
            url: format!("https://kitsune.example/users/{username}").as_str(),
            created_at: None,
        })
        .returning(Account::as_returning())
        .get_result(conn)
        .await
        .map_err(Into::into)
}

async fn create_user(conn: &mut AsyncPgConnection, username: &str) -> Result<User> {
    diesel::insert_into(users::table)
        .values(NewUser {
            id: Uuid::now_v7(),
            oidc_id: None,
            username,
            email: format!("{username}@kitsune.example").as_str(),
            password: None,
            confirmation_token: Uuid::now_v7().to_string().as_str(),
        })
        .returning(User::as_returning())
        .get_result(conn)
        .await
        .map_err(Into::into)
}

#[tokio::test]
async fn accounts_username() {
    database_test(async |db_pool| {
        with_connection_panicky!(db_pool, |conn| {
            let initial_insert = create_account(conn, "aumetra").await;
            initial_insert.unwrap();

            let case_mutation = create_account(conn, "AuMeTrA").await;
            case_mutation.unwrap_err();

            let unicode_mutation_1 = create_account(conn, "äumeträ").await;
            unicode_mutation_1.unwrap_err();

            let unicode_mutation_2 = create_account(conn, "🅰umetr🅰").await;
            unicode_mutation_2.unwrap_err();

            let unicode_case_mutation = create_account(conn, "🅰UMETR🅰").await;
            unicode_case_mutation.unwrap_err();
        });
    })
    .await;
}

#[tokio::test]
async fn users_username() {
    database_test(async |db_pool| {
        with_connection_panicky!(db_pool, |conn| {
            let initial_insert = create_user(conn, "aumetra").await;
            initial_insert.unwrap();

            let case_mutation = create_user(conn, "AuMeTrA").await;
            case_mutation.unwrap_err();

            let unicode_mutation_1 = create_user(conn, "äumeträ").await;
            unicode_mutation_1.unwrap_err();

            let unicode_mutation_2 = create_user(conn, "🅰umetr🅰").await;
            unicode_mutation_2.unwrap_err();

            let unicode_case_mutation = create_user(conn, "🅰UMETR🅰").await;
            unicode_case_mutation.unwrap_err();
        });
    })
    .await;
}
