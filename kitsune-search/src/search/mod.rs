use crate::config::Configuration;
use tantivy::{
    schema::{Field, Schema, INDEXED, TEXT},
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
    let id = schema.add_bytes_field("id", INDEXED);
    let data = schema.add_text_field("data", TEXT);
    let schema = schema.build();

    let search_schema = SearchSchema {
        id,
        data,
        tantivy_schema: schema.clone(),
    };
    let index = Index::create_in_dir(&config.index_dir_path, schema)?;

    Ok(SearchIndex {
        index,
        schema: search_schema,
    })
}
