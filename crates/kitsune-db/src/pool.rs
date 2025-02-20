#[macro_export]
macro_rules! with_connection {
    ($pool:expr_2021, |$conn_name:ident| $code:block) => {{
        let mut conn = $pool.get().await?;
        let $conn_name = &mut *conn;
        async { $code }.await
    }};
}

#[macro_export]
macro_rules! with_connection_panicky {
    ($pool:expr_2021, $($other:tt)*) => {{
        let result: ::std::result::Result<_, $crate::diesel_async::pooled_connection::bb8::RunError> = $crate::trials::attempt! { async
            $crate::with_connection!($pool, $($other)*)
        };
        result.unwrap()
    }};
}

#[macro_export]
macro_rules! with_transaction {
    ($pool:expr_2021, |$conn_name:ident| $code:block) => {{
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
