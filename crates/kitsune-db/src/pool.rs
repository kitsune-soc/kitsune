use diesel_async::{
    pooled_connection::deadpool::{Object, Pool, PoolError},
    scoped_futures::ScopedBoxFuture,
    AsyncConnection, AsyncPgConnection,
};
use std::future::Future;

#[derive(Clone)]
pub struct PgPool {
    inner: Pool<AsyncPgConnection>,
}

impl PgPool {
    /// Run the code inside a context with a database connection
    pub async fn with_connection<F, Fut, T, E>(&self, func: F) -> Result<T, E>
    where
        // Yes, this is *technically* leaky since a user could just move the object out of the closure
        // Just don't. kthx.
        F: FnOnce(Object<AsyncPgConnection>) -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: From<PoolError>,
    {
        let conn = self.inner.get().await?;
        func(conn).await
    }

    /// Run the code inside a context with a database transaction
    pub async fn with_transaction<'a, R, E, F>(&self, func: F) -> Result<R, E>
    where
        F: for<'r> FnOnce(
                &'r mut Object<AsyncPgConnection>,
            ) -> ScopedBoxFuture<'a, 'r, Result<R, E>>
            + Send
            + 'a,
        E: From<diesel::result::Error> + From<PoolError> + Send + 'a,
        R: Send + 'a,
    {
        let mut conn = self.inner.get().await?;
        conn.transaction(func).await
    }
}

impl From<Pool<AsyncPgConnection>> for PgPool {
    fn from(value: Pool<AsyncPgConnection>) -> Self {
        Self { inner: value }
    }
}
