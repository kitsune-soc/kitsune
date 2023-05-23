use diesel_async::RunQueryDsl;
use futures_util::Future;
use kitsune_db::PgPool;
use std::env;

pub async fn database_test<F, Fut>(func: F) -> Fut::Output
where
    F: FnOnce(PgPool) -> Fut,
    Fut: Future,
{
    let db_url = env::var("DATABASE_URL").expect("Missing database URL");
    let pool = kitsune_db::connect(&db_url, 10)
        .await
        .expect("Failed to connect to database");

    let out = func(pool.clone()).await;

    let mut db_conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    diesel::sql_query("DROP SCHEMA public CASCADE")
        .execute(&mut db_conn)
        .await
        .expect("Failed to delete schema");

    diesel::sql_query("CREATE SCHEMA public")
        .execute(&mut db_conn)
        .await
        .expect("Failed to create schema");

    out
}
