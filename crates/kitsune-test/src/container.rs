use testcontainers::{core::ContainerAsync, runners::AsyncRunner, RunnableImage};
use testcontainers_modules::{minio::MinIO, postgres::Postgres, redis::Redis};

pub trait Service {
    const PORT: u16;

    async fn url(&self) -> String;
}

impl Service for ContainerAsync<MinIO> {
    const PORT: u16 = 9000;

    async fn url(&self) -> String {
        let port = self.get_host_port_ipv4(Self::PORT).await;
        format!("http://127.0.0.1:{port}")
    }
}

impl Service for ContainerAsync<Postgres> {
    const PORT: u16 = 5432;

    async fn url(&self) -> String {
        let port = self.get_host_port_ipv4(Self::PORT).await;
        format!("postgres://postgres:postgres@127.0.0.1:{port}/test_db")
    }
}

impl Service for ContainerAsync<Redis> {
    const PORT: u16 = 6379;

    async fn url(&self) -> String {
        let port = self.get_host_port_ipv4(Self::PORT).await;
        format!("redis://127.0.0.1:{port}")
    }
}

pub async fn minio() -> impl Service {
    MinIO::default().start().await
}

pub async fn postgres() -> impl Service {
    let base = Postgres::default()
        .with_user("postgres")
        .with_password("postgres")
        .with_db_name("test_db");

    RunnableImage::from(base)
        .with_tag("15-alpine")
        .start()
        .await
}

pub async fn redis() -> impl Service {
    #[allow(clippy::default_constructed_unit_structs)]
    Redis::default().start().await
}
