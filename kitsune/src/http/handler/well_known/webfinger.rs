use crate::{error::Result, state::Zustand};
use axum::{
    extract::{Query, State},
    routing, Json, Router,
};
use axum_extra::either::Either;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use http::StatusCode;
use kitsune_core::service::url::UrlService;
use kitsune_db::{model::account::Account, schema::accounts, PgPool};
use kitsune_type::webfinger::{Link, Resource};
use scoped_futures::ScopedFutureExt;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
struct WebfingerQuery {
    resource: String,
}

#[utoipa::path(
    get,
    path = "/.well-known/webfinger",
    params(WebfingerQuery),
    responses(
        (status = 200, description = "Response with the location of the user's profile", body = Resource),
        (status = StatusCode::NOT_FOUND, description = "The service doesn't know this user"),
    )
)]
async fn get(
    State(db_pool): State<PgPool>,
    State(url_service): State<UrlService>,
    Query(query): Query<WebfingerQuery>,
) -> Result<Either<Json<Resource>, StatusCode>> {
    let username_at_instance = query.resource.trim_start_matches("acct:");
    let Some((username, instance)) = username_at_instance.split_once('@') else {
        return Ok(Either::E2(StatusCode::BAD_REQUEST));
    };

    let subject = if instance == url_service.webfinger_domain() {
        query.resource.clone()
    } else if instance == url_service.domain() {
        // Canonicalize the domain
        url_service.acct_uri(username)
    } else {
        return Ok(Either::E2(StatusCode::NOT_FOUND));
    };

    let account = db_pool
        .with_connection(|db_conn| {
            accounts::table
                .filter(
                    accounts::username
                        .eq(username)
                        .and(accounts::local.eq(true)),
                )
                .select(Account::as_select())
                .first::<Account>(db_conn)
                .scoped()
        })
        .await?;
    let account_url = url_service.user_url(account.id);

    Ok(Either::E1(Json(Resource {
        subject,
        aliases: vec![account_url.clone()],
        links: vec![Link {
            rel: "self".into(),
            r#type: Some("application/activity+json".into()),
            href: Some(account_url),
        }],
    })))
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}

#[cfg(test)]
mod tests {
    use super::{get, WebfingerQuery};
    use crate::error::Error;
    use axum::{
        extract::{Query, State},
        Json,
    };
    use axum_extra::either::Either;
    use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
    use http::StatusCode;
    use kitsune_core::service::url::UrlService;
    use kitsune_db::{
        model::account::{ActorType, NewAccount},
        schema::accounts,
    };
    use kitsune_test::database_test;
    use kitsune_type::webfinger::Link;
    use scoped_futures::ScopedFutureExt;
    use speedy_uuid::Uuid;

    #[tokio::test]
    #[serial_test::serial]
    async fn basic() {
        database_test(|db_pool| async move {
            let account_id = db_pool
                .with_connection(|db_conn| {
                    async move { Ok::<_, eyre::Report>(prepare_db(db_conn).await) }.scoped()
                })
                .await
                .unwrap();
            let account_url = format!("https://example.com/users/{account_id}");

            let db_conn = State(db_pool);
            let url_service = UrlService::builder()
                .scheme("https")
                .domain("example.com")
                .build();
            let url_service = State(url_service);

            // Should resolve a local user
            let query = WebfingerQuery {
                resource: "acct:alice@example.com".into(),
            };
            let response = get(db_conn.clone(), url_service.clone(), Query(query))
                .await
                .unwrap();
            let resource = match response {
                Either::E1(Json(resource)) => resource,
                Either::E2(status) => panic!("Unexpected status code: {status}"),
            };

            assert_eq!(resource.subject, "acct:alice@example.com");
            assert_eq!(resource.aliases, [account_url.clone()]);

            let [Link { rel, r#type, href }] = <[_; 1]>::try_from(resource.links).unwrap();

            assert_eq!(rel, "self");
            assert_eq!(r#type.unwrap(), "application/activity+json");
            assert_eq!(href.unwrap(), account_url);

            // Should respond with 404 for an unknown user
            let query = WebfingerQuery {
                resource: "acct:alice@example.net".into(),
            };
            let response = get(db_conn.clone(), url_service.clone(), Query(query))
                .await
                .unwrap();

            assert!(matches!(response, Either::E2(StatusCode::NOT_FOUND)));

            // Should not resolve a remote account
            let query = WebfingerQuery {
                resource: "acct:bob@example.net".into(),
            };
            let response = get(db_conn, url_service, Query(query)).await.unwrap();

            assert!(matches!(response, Either::E2(StatusCode::NOT_FOUND)));
        })
        .await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn custom_domain() {
        database_test(|db_pool| async move {
            db_pool
                .with_connection(|db_conn| {
                    async move {
                        prepare_db(db_conn).await;
                        Ok::<_, eyre::Report>(())
                    }
                    .scoped()
                })
                .await
                .unwrap();

            let db_pool = State(db_pool);
            let url_service = UrlService::builder()
                .scheme("https")
                .domain("example.com")
                .webfinger_domain(Some("alice.example".into()))
                .build();
            let url_service = State(url_service);

            // Should canonicalize the domain
            let query = WebfingerQuery {
                resource: "acct:alice@example.com".into(),
            };
            let response = get(db_pool.clone(), url_service.clone(), Query(query))
                .await
                .unwrap();
            let resource = match response {
                Either::E1(Json(resource)) => resource,
                Either::E2(status) => panic!("Unexpected status code: {status}"),
            };

            assert_eq!(resource.subject, "acct:alice@alice.example");

            // Should return the canonical domain as-is
            let query = WebfingerQuery {
                resource: "acct:alice@alice.example".into(),
            };
            let response = get(db_pool, url_service, Query(query)).await.unwrap();
            let resource = match response {
                Either::E1(Json(resource)) => resource,
                Either::E2(status) => panic!("Unexpected status code: {status}"),
            };

            assert_eq!(resource.subject, "acct:alice@alice.example");
        })
        .await;
    }

    async fn prepare_db(db_conn: &mut AsyncPgConnection) -> Uuid {
        // Create a local user `@alice` and a remote account `@bob`
        db_conn
            .transaction(|tx| {
                async move {
                    let account_id = Uuid::now_v7();
                    diesel::insert_into(accounts::table)
                        .values(NewAccount {
                            id: account_id,
                            display_name: None,
                            username: "alice",
                            locked: false,
                            note: None,
                            local: true,
                            domain: "example.com",
                            actor_type: ActorType::Person,
                            url: "https://example.com/users/alice",
                            featured_collection_url: None,
                            followers_url: None,
                            following_url: None,
                            inbox_url: None,
                            outbox_url: None,
                            shared_inbox_url: None,
                            public_key_id: "https://example.com/users/alice#main-key",
                            public_key: "",
                            created_at: None,
                        })
                        .execute(tx)
                        .await?;

                    diesel::insert_into(accounts::table)
                        .values(NewAccount {
                            id: Uuid::now_v7(),
                            display_name: None,
                            username: "bob",
                            locked: false,
                            note: None,
                            local: false,
                            domain: "example.net",
                            actor_type: ActorType::Person,
                            url: "https://example.net/users/bob",
                            featured_collection_url: None,
                            followers_url: None,
                            following_url: None,
                            inbox_url: None,
                            outbox_url: None,
                            shared_inbox_url: None,
                            public_key_id: "https://example.net/users/bob#main-key",
                            public_key: "",
                            created_at: None,
                        })
                        .execute(tx)
                        .await?;
                    Ok::<_, Error>(account_id)
                }
                .scope_boxed()
            })
            .await
            .unwrap()
    }
}
