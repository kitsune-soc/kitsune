pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20221113_181919_create_jobs_table;
mod m20221119_181259_create_tokens;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20221113_181919_create_jobs_table::Migration),
            Box::new(m20221119_181259_create_tokens::Migration),
        ]
    }
}
