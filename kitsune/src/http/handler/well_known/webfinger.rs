use crate::state::Zustand;
use axum::{
    extract::{Query, State},
    routing, Json, Router,
};
use axum_extra::either::Either;
use http::StatusCode;
use kitsune_error::Result;
use kitsune_service::account::{AccountService, GetUser};
use kitsune_type::webfinger::{Link, Resource};
use kitsune_url::UrlService;
use serde::Deserialize;

#[derive(Deserialize)]
struct WebfingerQuery {
    resource: String,
}

async fn get(
    State(account_service): State<AccountService>,
    State(url_service): State<UrlService>,
    Query(query): Query<WebfingerQuery>,
) -> Result<Either<Json<Resource>, StatusCode>> {
    let username_at_instance = query.resource.trim_start_matches("acct:");
    let Some((username, instance)) = username_at_instance.split_once('@') else {
        return Ok(Either::E2(StatusCode::BAD_REQUEST));
    };

    let get_user = GetUser::builder().username(username).build();
    let Some(account) = account_service.get(get_user).await? else {
        return Ok(Either::E2(StatusCode::NOT_FOUND));
    };
    let account_url = url_service.user_url(account.id);

    let subject = if instance == url_service.webfinger_domain() || instance == url_service.domain()
    {
        url_service.acct_uri(&account.username)
    } else {
        return Ok(Either::E2(StatusCode::NOT_FOUND));
    };

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
    use athena::{Coerce, RedisJobQueue};
    use axum::{
        extract::{Query, State},
        Json,
    };
    use axum_extra::either::Either;
    use bytes::Bytes;
    use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
    use fred::clients::RedisPool;
    use http::{Request, Response, StatusCode};
    use http_body_util::Empty;
    use kitsune_activitypub::Fetcher;
    use kitsune_cache::NoopCache;
    use kitsune_config::instance::FederationFilterConfiguration;
    use kitsune_core::traits::coerce::{CoerceFetcher, CoerceResolver};
    use kitsune_db::{
        model::account::{ActorType, NewAccount},
        schema::accounts,
        with_connection_panicky, PgPool,
    };
    use kitsune_error::Error;
    use kitsune_federation_filter::FederationFilter;
    use kitsune_http_client::Client;
    use kitsune_jobs::KitsuneContextRepo;
    use kitsune_search::NoopSearchService;
    use kitsune_service::{
        account::AccountService, attachment::AttachmentService, job::JobService,
    };
    use kitsune_storage::fs::Storage;
    use kitsune_test::{database_test, language_detection_config, redis_test};
    use kitsune_type::webfinger::Link;
    use kitsune_url::UrlService;
    use kitsune_webfinger::Webfinger;
    use scoped_futures::ScopedFutureExt;
    use speedy_uuid::Uuid;
    use std::convert::Infallible;
    use tempfile::TempDir;
    use tower::service_fn;
    use triomphe::Arc;

    async fn handle(
        _req: Request<kitsune_http_client::Body>,
    ) -> Result<Response<Empty<Bytes>>, Infallible> {
        Ok::<_, Infallible>(Response::new(Empty::new()))
    }

    fn build_account_service(
        db_pool: PgPool,
        redis_pool: RedisPool,
        url_service: UrlService,
    ) -> AccountService {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_owned());
        let client = Client::builder().service(service_fn(handle));

        let attachment_service = AttachmentService::builder()
            .client(client.clone())
            .db_pool(db_pool.clone())
            .url_service(url_service.clone())
            .storage_backend(storage)
            .media_proxy_enabled(false)
            .build();

        let resolver = Arc::new(Webfinger::new(Arc::new(NoopCache.into()))).coerce();

        let fetcher = Fetcher::builder()
            .client(client)
            .db_pool(db_pool.clone())
            .embed_client(None)
            .federation_filter(
                FederationFilter::new(&FederationFilterConfiguration::Deny {
                    domains: Vec::new(),
                })
                .unwrap(),
            )
            .language_detection_config(language_detection_config())
            .search_backend(NoopSearchService)
            .account_cache(Arc::new(NoopCache.into()))
            .post_cache(Arc::new(NoopCache.into()))
            .resolver(resolver.clone())
            .build()
            .coerce();

        let context_repo = KitsuneContextRepo::builder()
            .db_pool(db_pool.clone())
            .build();
        let job_queue = RedisJobQueue::builder()
            .context_repository(context_repo)
            .queue_name("webfinger_test")
            .redis_pool(redis_pool)
            .build();

        let job_service = JobService::builder()
            .job_queue(Arc::new(job_queue).coerce())
            .build();

        AccountService::builder()
            .attachment_service(attachment_service)
            .db_pool(db_pool)
            .fetcher(fetcher)
            .job_service(job_service)
            .url_service(url_service)
            .resolver(resolver)
            .build()
    }

    #[tokio::test]
    async fn basic() {
        database_test(|db_pool| {
            redis_test(|redis_pool| async move {
                let account_id =
                    with_connection_panicky!(db_pool, |db_conn| { prepare_db(db_conn).await });
                let account_url = format!("https://example.com/users/{account_id}");

                let url_service = UrlService::builder()
                    .scheme("https")
                    .domain("example.com")
                    .build();
                let account_service =
                    build_account_service(db_pool, redis_pool, url_service.clone());

                let account_service = State(account_service);
                let url_service = State(url_service);

                // Should resolve a local user
                let query = WebfingerQuery {
                    resource: "acct:alice@example.com".into(),
                };
                let response = get(account_service.clone(), url_service.clone(), Query(query))
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
                let response = get(account_service.clone(), url_service.clone(), Query(query))
                    .await
                    .unwrap();

                assert!(matches!(response, Either::E2(StatusCode::NOT_FOUND)));

                // Should not resolve a remote account
                let query = WebfingerQuery {
                    resource: "acct:bob@example.net".into(),
                };
                let response = get(account_service, url_service, Query(query))
                    .await
                    .unwrap();

                assert!(matches!(response, Either::E2(StatusCode::NOT_FOUND)));
            })
        })
        .await;
    }

    #[tokio::test]
    async fn custom_domain() {
        database_test(|db_pool| {
            redis_test(|redis_pool| async move {
                with_connection_panicky!(db_pool, |db_conn| {
                    prepare_db(db_conn).await;
                });

                let url_service = UrlService::builder()
                    .scheme("https")
                    .domain("example.com")
                    .webfinger_domain(Some("alice.example".into()))
                    .build();
                let account_service =
                    build_account_service(db_pool, redis_pool, url_service.clone());

                let account_service = State(account_service);
                let url_service = State(url_service);

                // Should canonicalize the domain
                let query = WebfingerQuery {
                    resource: "acct:alice@example.com".into(),
                };
                let response = get(account_service.clone(), url_service.clone(), Query(query))
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
                let response = get(account_service, url_service, Query(query))
                    .await
                    .unwrap();
                let resource = match response {
                    Either::E1(Json(resource)) => resource,
                    Either::E2(status) => panic!("Unexpected status code: {status}"),
                };

                assert_eq!(resource.subject, "acct:alice@alice.example");
            })
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
