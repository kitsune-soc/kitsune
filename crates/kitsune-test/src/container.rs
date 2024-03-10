use testcontainers::{clients::Cli as CliClient, Container};
use testcontainers_modules::{postgres::Postgres, redis::Redis};

pub trait Service {
    const PORT: u16;

    fn url(&self) -> String;
}

impl Service for Container<'_, Postgres> {
    const PORT: u16 = 5432;

    fn url(&self) -> String {
        let port = self.get_host_port_ipv4(Self::PORT);
        format!("postgres://postgres:postgres@localhost:{port}/test_db")
    }
}

impl Service for Container<'_, Redis> {
    const PORT: u16 = 6379;

    fn url(&self) -> String {
        let port = self.get_host_port_ipv4(Self::PORT);
        format!("redis://localhost:{port}")
    }
}

pub fn postgres(client: &CliClient) -> Container<'_, Postgres> {
    client.run(
        Postgres::default()
            .with_user("postgres")
            .with_password("postgres")
            .with_db_name("test_db"),
    )
}

pub fn redis(client: &CliClient) -> Container<'_, Redis> {
    #[allow(clippy::default_constructed_unit_structs)]
    client.run(Redis::default())
}
