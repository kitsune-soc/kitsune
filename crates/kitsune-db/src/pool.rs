#[macro_export]
macro_rules! with_connection {
    ($pool:expr, |$conn_name:ident| $code:block) => {{
        let mut conn = $pool.get().await?;
        let $conn_name = &mut *conn;
        async { $code }.await
    }};
}

#[macro_export]
macro_rules! catch_error {
    ($($tt:tt)*) => {{
        let result: ::std::result::Result<_, ::diesel_async::pooled_connection::bb8::RunError> = async {
            Ok({ $($tt)* })
        }.await;
        result
    }};
}

#[macro_export]
macro_rules! with_connection_panicky {
    ($pool:expr, $($other:tt)*) => {{
        $crate::catch_error!($crate::with_connection!($pool, $($other)*)).unwrap()
    }};
}

#[macro_export]
macro_rules! with_transaction {
    ($pool:expr, |$conn_name:ident| $code:block) => {{
        use $crate::diesel_async::AsyncConnection;

        let mut conn = $pool.get().await?;
        conn.transaction(|conn| {
            Box::pin(async move {
                let $conn_name = conn;
                $code
            })
        })
        .await
    }};
}
