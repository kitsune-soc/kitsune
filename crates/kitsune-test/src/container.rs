use testcontainers::{clients::Cli as CliClient, Container, RunnableImage};
use testcontainers_modules::{minio::MinIO, postgres::Postgres, redis::Redis};

pub trait Service {
    const PORT: u16;

    fn url(&self) -> String;
}

impl Service for Container<'_, MinIO> {
    const PORT: u16 = 9000;

    fn url(&self) -> String {
        let port = self.get_host_port_ipv4(Self::PORT);
        format!("http://localhost:{port}")
    }
}

impl Service for Container<'_, Postgres> {
    const PORT: u16 = 5432;

    fn url(&self) -> String {
        let port = self.get_host_port_ipv4(Self::PORT);
        format!("postgres://postgres:postgres@127.0.0.1:{port}/test_db")
    }
}

impl Service for Container<'_, Redis> {
    const PORT: u16 = 6379;

    fn url(&self) -> String {
        let port = self.get_host_port_ipv4(Self::PORT);
        format!("redis://127.0.0.1:{port}")
    }
}

pub fn minio(client: &CliClient) -> impl Service + '_ {
    client.run(MinIO::default())
}

pub fn postgres(client: &CliClient) -> impl Service + '_ {
    let base = Postgres::default()
        .with_user("postgres")
        .with_password("postgres")
        .with_db_name("test_db");

    client.run(RunnableImage::from(base).with_tag("15-alpine"))
}

pub fn redis(client: &CliClient) -> impl Service + '_ {
    #[allow(clippy::default_constructed_unit_structs)]
    client.run(Redis::default())
}
