use kitsune_search::{config::Configuration, search::SearchIndex};
use kitsune_search_proto::{index::index_client::IndexClient, search::search_client::SearchClient};
use rand::Rng;
use tempfile::TempDir;
use tonic::transport::Channel;

pub struct TestClient {
    pub index: IndexClient<Channel>,
    pub search: SearchClient<Channel>,
    _temp_dir: TempDir,
}

impl TestClient {
    pub async fn create() -> Self {
        let port = rand::thread_rng().gen_range(1025..u16::MAX);
        let temp_dir = TempDir::new().unwrap();

        let config = Configuration {
            index_dir_path: temp_dir.path().into(),
            levenshtein_distance: 2,
            memory_arena_size: "3MB".parse().unwrap(),
            port,
            prometheus_port: 0, // Doesn't matter. No Prometheus server gets started
            read_only: false,
        };

        let search_index = SearchIndex::prepare(&config).unwrap();
        tokio::spawn(kitsune_search::grpc::start(config, search_index));

        Self {
            index: IndexClient::connect(format!("http://localhost:{port}"))
                .await
                .unwrap(),
            search: SearchClient::connect(format!("http://localhost:{port}"))
                .await
                .unwrap(),
            _temp_dir: temp_dir,
        }
    }
}
