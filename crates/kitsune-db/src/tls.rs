use diesel::{ConnectionError, ConnectionResult};
use diesel_async::{AsyncPgConnection, pooled_connection::ManagerConfig};
use futures_util::{FutureExt, future::BoxFuture};
use rustls_platform_verifier::BuilderVerifierExt;

pub fn pool_config() -> ManagerConfig<AsyncPgConnection> {
    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_conn);
    config
}

fn establish_conn(config: &str) -> BoxFuture<'_, ConnectionResult<AsyncPgConnection>> {
    async {
        let rustls_config = rustls::ClientConfig::builder()
            .with_platform_verifier()
            .map_err(|err| {
                error!(error = ?err);
                ConnectionError::BadConnection(err.to_string())
            })?
            .with_no_client_auth();

        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
        let (client, conn) = tokio_postgres::connect(config, tls).await.map_err(|err| {
            error!(error = ?err);
            ConnectionError::BadConnection(err.to_string())
        })?;

        tokio::spawn(async move {
            if let Err(err) = conn.await {
                error!("Database connection error: {err}");
            }
        });

        AsyncPgConnection::try_from(client).await
    }
    .boxed()
}
