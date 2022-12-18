pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20221113_181919_create_jobs_table;
mod m20221121_210920_create_oauth2_tables;
mod m20221127_194924_create_media_attachment_tables;
mod m20221127_211534_create_follow_table;
mod m20221213_214258_create_mention_table;
mod m20221213_215428_create_favourites_table;
mod m20221213_220058_create_repost_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20221113_181919_create_jobs_table::Migration),
            Box::new(m20221121_210920_create_oauth2_tables::Migration),
            Box::new(m20221127_194924_create_media_attachment_tables::Migration),
            Box::new(m20221127_211534_create_follow_table::Migration),
            Box::new(m20221213_214258_create_mention_table::Migration),
            Box::new(m20221213_215428_create_favourites_table::Migration),
            Box::new(m20221213_220058_create_repost_table::Migration),
        ]
    }
}
