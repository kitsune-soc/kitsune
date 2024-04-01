#[macro_export]
macro_rules! with_connection {
    ($pool:expr, |$conn_name:ident| $code:block) => {{
        let mut conn = $pool.get().await?;
        let $conn_name = &mut *conn;
        async move { $code }.await
    }};
}

#[macro_export]
macro_rules! with_connection_panicky {
    ($pool:expr, $($other:tt)*) => {{
        let result: ::std::result::Result<_, Box<dyn ::std::error::Error>> = async move {
            let _ = $crate::with_connection!($pool, $($other)*);
            Ok(())
        }.await;
        result.unwrap();
    }};
}

#[macro_export]
macro_rules! with_transaction {
    ($pool:expr, $func:expr) => {{
        let mut conn = $pool.get().await?;
        conn.transaction(|conn| Box::pin(($func)(conn))).await
    }};
}
