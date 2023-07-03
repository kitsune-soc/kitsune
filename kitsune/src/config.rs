use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::path::Path;

#[derive(Clone, Deserialize, Serialize)]
pub struct RedisCacheConfiguration {
    pub redis_url: SmolStr,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum CacheConfiguration {
    Redis(RedisCacheConfiguration),
    InMemory,
    None,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct DatabaseConfiguration {
    pub url: SmolStr,
    pub max_connections: u32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct EmbedConfiguration {
    pub url: SmolStr,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum FederationFilterConfiguration {
    Allow { domains: Vec<SmolStr> },
    Deny { domains: Vec<SmolStr> },
}

#[derive(Clone, Deserialize, Serialize)]
pub struct InstanceConfiguration {
    pub name: SmolStr,
    pub description: SmolStr,
    pub character_limit: usize,
    pub federation_filter: FederationFilterConfiguration,
    pub registrations_open: bool,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct JobQueueConfiguration {
    pub redis_url: SmolStr,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct OidcConfiguration {
    pub server_url: SmolStr,
    pub client_id: SmolStr,
    pub client_secret: SmolStr,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RedisMessagingConfiguration {
    pub redis_url: SmolStr,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum MessagingConfiguration {
    Redis(RedisMessagingConfiguration),
    InProcess,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct KitsuneSearchConfiguration {
    pub index_server: SmolStr,
    pub search_servers: Vec<SmolStr>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MeiliSearchConfiguration {
    pub instance_url: SmolStr,
    pub api_key: SmolStr,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum SearchConfiguration {
    Kitsune(KitsuneSearchConfiguration),
    Meilisearch(MeiliSearchConfiguration),
    Sql,
    None,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ServerConfiguration {
    pub frontend_dir: SmolStr,
    pub job_workers: usize,
    pub max_upload_size: usize,
    pub media_proxy_enabled: bool,
    pub oidc: Option<OidcConfiguration>,
    pub port: u16,
    pub prometheus_port: u16,
    pub request_timeout_sec: u64,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct FsStorageConfiguration {
    pub upload_dir: SmolStr,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct S3StorageConfiguration {
    pub bucket_name: SmolStr,
    pub endpoint_url: SmolStr,
    pub region: SmolStr,
    pub force_path_style: bool,
    pub access_key: SmolStr,
    pub secret_access_key: SmolStr,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum StorageConfiguration {
    Fs(FsStorageConfiguration),
    S3(S3StorageConfiguration),
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UrlConfiguration {
    pub scheme: SmolStr,
    pub domain: SmolStr,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Configuration {
    pub cache: CacheConfiguration,
    pub database: DatabaseConfiguration,
    pub embed: Option<EmbedConfiguration>,
    pub instance: InstanceConfiguration,
    pub job_queue: JobQueueConfiguration,
    pub messaging: MessagingConfiguration,
    pub server: ServerConfiguration,
    pub search: SearchConfiguration,
    pub storage: StorageConfiguration,
    pub url: UrlConfiguration,
}

impl Configuration {
    #[allow(clippy::result_large_err)] // This function is called once. It's fine if it's a little large.
    pub fn load<P>(path: P) -> serde_dhall::Result<Self>
    where
        P: AsRef<Path>,
    {
        serde_dhall::from_file(path)
            //.static_type_annotation() // SmolStr usage makes this impossible (unfortunately)
            .parse()
    }
}
