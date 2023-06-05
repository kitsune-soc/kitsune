//!
//! gRPC service
//!

use self::service::{IndexService, SearchService};
use crate::{config::Configuration, search::SearchIndex};
use kitsune_search_proto::{index::index_server::IndexServer, search::search_server::SearchServer};
use tokio::sync::RwLock;
use tonic::transport::Server;
use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};

pub mod service;

/// Start the search (and possibly index) gRPC service
#[instrument(skip_all)]
pub async fn start(config: Configuration, search_index: SearchIndex) {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<IndexServer<IndexService>>()
        .await;
    health_reporter
        .set_serving::<SearchServer<SearchService>>()
        .await;

    let account_reader = search_index.indices.account.reader().unwrap();
    let post_reader = search_index.indices.post.reader().unwrap();

    let mut server = Server::builder()
        .layer(AddExtensionLayer::new(config.clone()))
        .layer(AddExtensionLayer::new(search_index.clone()))
        .layer(TraceLayer::new_for_grpc())
        .add_service(health_service);

    if !config.read_only {
        let account_writer = search_index
            .indices
            .account
            .writer(config.memory_arena_size.to_bytes() as usize)
            .unwrap();

        let post_writer = search_index
            .indices
            .post
            .writer(config.memory_arena_size.to_bytes() as usize)
            .unwrap();

        server = server.add_service(IndexServer::new(IndexService {
            account: RwLock::new(account_writer),
            post: RwLock::new(post_writer),
        }));
    }

    server
        .add_service(SearchServer::new(SearchService {
            account: account_reader,
            post: post_reader,
        }))
        .serve(([0, 0, 0, 0], config.port).into())
        .await
        .unwrap();
}
