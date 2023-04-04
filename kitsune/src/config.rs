use serde::{Deserialize, Serialize};
use serde_dhall::StaticType;
use std::path::Path;

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub struct RedisCacheConfiguration {
    pub redis_url: String,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub enum CacheConfiguration {
    Redis(RedisCacheConfiguration),
    InMemory,
    None,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub struct DatabaseConfiguration {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub enum FederationFilterConfiguration {
    Allow { domains: Vec<String> },
    Deny { domains: Vec<String> },
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub struct InstanceConfiguration {
    pub name: String,
    pub description: String,
    pub character_limit: usize,
    pub federation_filter: FederationFilterConfiguration,
    pub registrations_open: bool,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub struct OidcConfiguration {
    pub server_url: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub struct RedisMessagingConfiguration {
    pub redis_url: String,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub enum MessagingConfiguration {
    Redis(RedisMessagingConfiguration),
    InProcess,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub struct KitsuneSearchConfiguration {
    pub index_server: String,
    pub search_servers: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub struct MeiliSearchConfiguration {
    pub instance_url: String,
    pub api_key: String,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub enum SearchConfiguration {
    Kitsune(KitsuneSearchConfiguration),
    Meilisearch(MeiliSearchConfiguration),
    Sql,
    None,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub struct ServerConfiguration {
    pub frontend_dir: String,
    pub job_workers: usize,
    pub max_upload_size: usize,
    pub media_proxy_enabled: bool,
    pub oidc: Option<OidcConfiguration>,
    pub port: u16,
    pub prometheus_port: u16,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub struct FsStorageConfiguration {
    pub upload_dir: String,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub struct S3StorageConfiguration {
    pub bucket_name: String,
    pub endpoint_url: String,
    pub region: String,
    pub force_path_style: bool,
    pub access_key: String,
    pub secret_access_key: String,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub enum StorageConfiguration {
    Fs(FsStorageConfiguration),
    S3(S3StorageConfiguration),
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub struct UrlConfiguration {
    pub scheme: String,
    pub domain: String,
}

#[derive(Clone, Deserialize, Serialize, StaticType)]
pub struct Configuration {
    pub cache: CacheConfiguration,
    pub database: DatabaseConfiguration,
    pub instance: InstanceConfiguration,
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
            .static_type_annotation()
            .parse()
    }
}
