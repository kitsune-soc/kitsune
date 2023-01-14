use self::{
    proto::{index::index_server::IndexServer, search::search_server::SearchServer},
    service::{IndexService, SearchService},
};
use crate::config::Configuration;
use tonic::transport::Server;
use tower_http::trace::TraceLayer;

mod proto;
mod service;

#[instrument(skip_all)]
pub async fn start(config: Configuration) {
    Server::builder()
        .layer(TraceLayer::new_for_grpc())
        .add_service(IndexServer::new(IndexService))
        .add_service(SearchServer::new(SearchService))
        .serve(([0, 0, 0, 0], config.port).into())
        .await
        .unwrap();
}
