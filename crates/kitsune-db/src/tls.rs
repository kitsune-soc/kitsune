use diesel::{ConnectionError, ConnectionResult};
use diesel_async::{pooled_connection::ManagerConfig, AsyncPgConnection};
use futures_util::{future::BoxFuture, FutureExt};

pub fn pool_config() -> ManagerConfig<AsyncPgConnection> {
    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_conn);
    config
}

fn establish_conn(config: &str) -> BoxFuture<'_, ConnectionResult<AsyncPgConnection>> {
    async {
        let rustls_config = rustls::ClientConfig::builder()
            .with_root_certificates(load_certs())
            .with_no_client_auth();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|err| ConnectionError::BadConnection(err.to_string()))?;

        tokio::spawn(async move {
            if let Err(err) = conn.await {
                error!("Database connection error: {err}");
            }
        });

        AsyncPgConnection::try_from(client).await
    }
    .boxed()
}

fn load_certs() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();
    let certs =
        rustls_native_certs::load_native_certs().expect("Failed to load native certificates");
    roots.add_parsable_certificates(certs);
    roots
}
