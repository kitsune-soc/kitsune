use super::handle::handle;
use hyper::{Request, Uri};
use iso8601_timestamp::Timestamp;
use kitsune_activitypub::{fetcher::MAX_FETCH_DEPTH, Fetcher};
use kitsune_cache::NoopCache;
use kitsune_config::instance::FederationFilterConfiguration;
use kitsune_core::traits::Fetcher as _;
use kitsune_federation_filter::FederationFilter;
use kitsune_http_client::Client;
use kitsune_search::NoopSearchService;
use kitsune_test::{build_ap_response, database_test};
use kitsune_type::ap::{
    actor::{Actor, ActorType, PublicKey},
    ap_context, AttributedToField, Object, ObjectType, PUBLIC_IDENTIFIER,
};
use kitsune_webfinger::Webfinger;
use std::{
    convert::Infallible,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use tower::service_fn;

#[tokio::test]
#[serial_test::serial]
async fn fetch_infinitely_long_reply_chain() {
    database_test(|db_pool| async move {
            let request_counter = Arc::new(AtomicU32::new(0));
            let client = service_fn(move |req: Request<_>| {
                let count = request_counter.fetch_add(1, Ordering::SeqCst);
                assert!(MAX_FETCH_DEPTH * 3 >= count);

                async move {
                    let author_id = "https://example.com/users/1".to_owned();
                    let author = Actor {
                        context: ap_context(),
                        id: author_id.clone(),
                        r#type: ActorType::Person,
                        name: None,
                        preferred_username: "InfiniteNotes".into(),
                        subject: None,
                        icon: None,
                        image: None,
                        manually_approves_followers: false,
                        public_key: PublicKey {
                            id: format!("{author_id}#main-key"),
                            owner: author_id,
                            // A 512-bit RSA public key generated as a placeholder
                            public_key_pem: "-----BEGIN PUBLIC KEY-----\nMFwwDQYJKoZIhvcNAQEBBQADSwAwSAJBAK1v4oRbdBPi8oRL0M1GQqSWtkb9uE2L\nJCAgZK9KiVECNYvEASYor7DeMEu6BxR1E4XI2DlGkigClWXFhQDhos0CAwEAAQ==\n-----END PUBLIC KEY-----\n".into(),
                        },
                        endpoints: None,
                        featured: None,
                        inbox: "https://example.com/inbox".into(),
                        outbox: None,
                        followers: None,
                        following: None,
                        published: Timestamp::UNIX_EPOCH,
                    };

                    if let Some(note_id) = req.uri().path_and_query().unwrap().as_str().strip_prefix("/notes/") {
                        let note_id = note_id.parse::<u32>().unwrap();
                        let note = Object {
                            context: ap_context(),
                            id: format!("https://example.com/notes/{note_id}"),
                            r#type: ObjectType::Note,
                            attributed_to: AttributedToField::Url(author.id.clone()),
                            in_reply_to: Some(format!("https://example.com/notes/{}", note_id + 1)),
                            name: None,
                            summary: None,
                            content: String::new(),
                            media_type: None,
                            attachment: Vec::new(),
                            tag: Vec::new(),
                            sensitive: false,
                            published: Timestamp::UNIX_EPOCH,
                            to: vec![PUBLIC_IDENTIFIER.into()],
                            cc: Vec::new(),
                        };

                        let body = simd_json::to_string(&note).unwrap();

                        Ok::<_, Infallible>(build_ap_response(body))
                    } else if req.uri().path_and_query().unwrap() == Uri::try_from(&author.id).unwrap().path_and_query().unwrap() {
                        let body = simd_json::to_string(&author).unwrap();

                        Ok::<_, Infallible>(build_ap_response(body))
                    } else {
                        handle(req).await
                    }
                }
            });
            let client = Client::builder().service(client);

            let fetcher = Fetcher::builder()
                .client(client.clone())
                .db_pool(db_pool)
                .embed_client(None)
                .federation_filter(
                    FederationFilter::new(&FederationFilterConfiguration::Deny {
                        domains: Vec::new(),
                    })
                    .unwrap(),
                )
                .search_backend(NoopSearchService)
                .resolver(Arc::new(Webfinger::with_client(client, Arc::new(NoopCache.into()))))
                .account_cache(Arc::new(NoopCache.into()))
                .post_cache(Arc::new(NoopCache.into()))
                .build();

            assert!(fetcher
                .fetch_post("https://example.com/notes/0".into())
                .await
                .is_ok());
        })
        .await;
}
