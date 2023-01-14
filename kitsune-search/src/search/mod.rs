use std::fs;

use crate::config::Configuration;
use tantivy::{
    directory::MmapDirectory,
    schema::{Field, Schema, INDEXED, STORED, TEXT},
    Index,
};

mod schema;

#[derive(Clone)]
pub struct SearchIndex {
    pub index: Index,
    pub schema: SearchSchema,
}

#[derive(Clone)]
pub struct SearchSchema {
    pub id: Field,
    pub data: Field,
    pub tantivy_schema: Schema,
}

pub fn prepare_index(config: &Configuration) -> tantivy::Result<SearchIndex> {
    let mut schema = Schema::builder();
    let id = schema.add_bytes_field("id", INDEXED | STORED);
    let data = schema.add_text_field("data", TEXT);
    let schema = schema.build();

    let search_schema = SearchSchema {
        id,
        data,
        tantivy_schema: schema.clone(),
    };

    if !config.index_dir_path.exists() {
        fs::create_dir(&config.index_dir_path).ok();
    }

    let directory = MmapDirectory::open(&config.index_dir_path)?;
    let index = Index::open_or_create(directory, schema)?;

    Ok(SearchIndex {
        index,
        schema: search_schema,
    })
}
