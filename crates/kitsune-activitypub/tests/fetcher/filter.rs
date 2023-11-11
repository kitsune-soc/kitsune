#[tokio::test]
#[serial_test::serial]
async fn federation_allow() {
    database_test(|db_pool| async move {
        let builder = Fetcher::builder()
            .db_pool(db_pool)
            .embed_client(None)
            .federation_filter(
                FederationFilterService::new(&FederationFilterConfiguration::Allow {
                    domains: vec!["corteximplant.com".into()],
                })
                .unwrap(),
            )
            .search_backend(NoopSearchService)
            .post_cache(Arc::new(NoopCache.into()))
            .user_cache(Arc::new(NoopCache.into()));

        let client = service_fn(
            #[allow(unreachable_code)] // https://github.com/rust-lang/rust/issues/67227
            |_: Request<_>| async {
                panic!("Requested a denied domain") as Result<Response<Body>, Infallible>
            },
        );
        let client = Client::builder().service(client);
        let fetcher = builder
            .clone()
            .client(client.clone())
            .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
            .build();

        assert!(matches!(
            fetcher.fetch_object("https://example.com/fakeobject").await,
            Err(Error::Api(ApiError::Unauthorised))
        ));
        assert!(matches!(
            fetcher
                .fetch_object("https://other.badstuff.com/otherfake")
                .await,
            Err(Error::Api(ApiError::Unauthorised))
        ));

        let client = Client::builder().service(service_fn(handle));
        let fetcher = builder
            .clone()
            .client(client.clone())
            .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
            .build();

        assert!(matches!(
            fetcher
                .fetch_object("https://corteximplant.com/@0x0/109501674056556919")
                .await,
            Ok(..)
        ));
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn federation_deny() {
    database_test(|db_pool| async move {
        let client = service_fn(
            #[allow(unreachable_code)]
            |_: Request<_>| async {
                panic!("Requested a denied domain") as Result<Response<Body>, Infallible>
            },
        );
        let client = Client::builder().service(client);

        let fetcher = Fetcher::builder()
            .client(client.clone())
            .db_pool(db_pool)
            .embed_client(None)
            .federation_filter(
                FederationFilterService::new(&FederationFilterConfiguration::Deny {
                    domains: vec!["example.com".into(), "*.badstuff.com".into()],
                })
                .unwrap(),
            )
            .search_backend(NoopSearchService)
            .webfinger(Webfinger::with_client(client, Arc::new(NoopCache.into())))
            .post_cache(Arc::new(NoopCache.into()))
            .user_cache(Arc::new(NoopCache.into()))
            .build();

        assert!(matches!(
            fetcher.fetch_object("https://example.com/fakeobject").await,
            Err(Error::Api(ApiError::Unauthorised))
        ));
        assert!(matches!(
            fetcher
                .fetch_object("https://other.badstuff.com/otherfake")
                .await,
            Err(Error::Api(ApiError::Unauthorised))
        ));
    })
    .await;
}
