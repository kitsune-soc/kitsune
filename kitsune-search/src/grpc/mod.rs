use self::service::{IndexService, SearchService};
use crate::{config::Configuration, search::SearchIndex};
use kitsune_search_proto::{index::index_server::IndexServer, search::search_server::SearchServer};
use tokio::sync::Mutex;
use tonic::transport::Server;
use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};

pub mod service;

#[instrument(skip_all)]
pub async fn start(config: Configuration, search_index: SearchIndex) {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<IndexServer<IndexService>>()
        .await;
    health_reporter
        .set_serving::<SearchServer<SearchService>>()
        .await;

    let account_reader = search_index.indicies.account.reader().unwrap();
    let account_writer = search_index
        .indicies
        .account
        .writer(config.memory_arena_size.to_bytes() as usize)
        .unwrap();

    let post_reader = search_index.indicies.post.reader().unwrap();
    let post_writer = search_index
        .indicies
        .post
        .writer(config.memory_arena_size.to_bytes() as usize)
        .unwrap();

    Server::builder()
        .layer(AddExtensionLayer::new(config.clone()))
        .layer(AddExtensionLayer::new(search_index))
        .layer(TraceLayer::new_for_grpc())
        .add_service(health_service)
        .add_service(IndexServer::new(IndexService {
            account: Mutex::new(account_writer),
            post: Mutex::new(post_writer),
        }))
        .add_service(SearchServer::new(SearchService {
            account: account_reader,
            post: post_reader,
        }))
        .serve(([0, 0, 0, 0], config.port).into())
        .await
        .unwrap();
}
