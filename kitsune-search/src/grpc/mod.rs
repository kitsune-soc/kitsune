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
    let reader = search_index.index.reader().unwrap();
    let writer = search_index.index.writer(config.memory_arena_size).unwrap();

    Server::builder()
        .layer(AddExtensionLayer::new(search_index))
        .layer(TraceLayer::new_for_grpc())
        .add_service(IndexServer::new(IndexService {
            writer: Mutex::new(writer),
        }))
        .add_service(SearchServer::new(SearchService { reader }))
        .serve(([0, 0, 0, 0], config.port).into())
        .await
        .unwrap();
}
