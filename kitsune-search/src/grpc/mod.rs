use self::{
    proto::{index::index_server::IndexServer, search::search_server::SearchServer},
    service::{IndexService, SearchService},
};
use crate::{config::Configuration, search::SearchIndex};
use tokio::sync::Mutex;
use tonic::transport::Server;
use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};

mod proto;
mod service;

#[instrument(skip_all)]
pub async fn start(config: Configuration, search_index: SearchIndex) {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<IndexServer<IndexService>>()
        .await;
    health_reporter
        .set_serving::<SearchServer<SearchService>>()
        .await;

    let reader = search_index.index.reader().unwrap();
    let writer = search_index
        .index
        .writer(config.memory_arena_size.to_bytes() as usize)
        .unwrap();

    Server::builder()
        .layer(AddExtensionLayer::new(config.clone()))
        .layer(AddExtensionLayer::new(search_index))
        .layer(TraceLayer::new_for_grpc())
        .add_service(health_service)
        .add_service(IndexServer::new(IndexService {
            writer: Mutex::new(writer),
        }))
        .add_service(SearchServer::new(SearchService { reader }))
        .serve(([0, 0, 0, 0], config.port).into())
        .await
        .unwrap();
}
