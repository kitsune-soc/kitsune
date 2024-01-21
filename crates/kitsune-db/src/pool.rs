use diesel_async::{
    pooled_connection::deadpool::{Object, Pool, PoolError as DeadpoolError},
    scoped_futures::{ScopedBoxFuture, ScopedFutureWrapper},
    AsyncConnection, AsyncPgConnection,
};
use miette::Diagnostic;
use std::{
    fmt::{Debug, Display},
    future::Future,
};
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
pub enum PoolError<E>
where
    E: Display + Debug,
{
    #[error(transparent)]
    Pool(#[from] DeadpoolError),

    #[error("{0}")]
    User(E),
}

/// Small wrapper around [`Pool<AsyncPgConnection>`]
///
/// The intent of this API is to encourage and make short-lived ownership of connections easier.
/// With the traditional RAII guard based approach, it is rather hard (and/or ugly) to define clear scopes for connections
/// (especially when they are used *a lot* throughout the code).
///
/// The API of this wrapper is based on closures, meaning you have no choice but to be aware of the scope.
/// And the extra level of indentation this forces is supposed to coerce users to keep the scope as small as possible.
#[derive(Clone)]
pub struct PgPool {
    inner: Pool<AsyncPgConnection>,
}

impl PgPool {
    /// Run the code inside a context with a database connection
    pub async fn with_connection<'a, F, Fut, T, E>(&self, func: F) -> Result<T, PoolError<E>>
    where
        for<'conn> F:
            FnOnce(&'conn mut Object<AsyncPgConnection>) -> ScopedFutureWrapper<'conn, 'a, Fut>,
        Fut: Future<Output = Result<T, E>>,
        E: Display + Debug,
    {
        let mut conn = self.inner.get().await?;
        func(&mut conn).await.map_err(PoolError::User)
    }

    /// Run the code inside a context with a database transaction
    pub async fn with_transaction<'a, R, E, F>(&self, func: F) -> Result<R, PoolError<E>>
    where
        F: for<'r> FnOnce(
                &'r mut Object<AsyncPgConnection>,
            ) -> ScopedBoxFuture<'a, 'r, Result<R, E>>
            + Send
            + 'a,
        E: From<diesel::result::Error> + Debug + Display + Send + 'a,
        R: Send + 'a,
    {
        let mut conn = self.inner.get().await?;
        conn.transaction(func).await.map_err(PoolError::User)
    }
}

impl From<Pool<AsyncPgConnection>> for PgPool {
    fn from(value: Pool<AsyncPgConnection>) -> Self {
        Self { inner: value }
    }
}
