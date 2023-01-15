use std::fs;

use self::schema::{AccountSchema, PostSchema};
use crate::config::Configuration;
use tantivy::{directory::MmapDirectory, Index};

pub mod schema;

#[derive(Clone)]
pub struct SearchIndicies {
    pub account: Index,
    pub post: Index,
}

#[derive(Clone, Default)]
pub struct SearchSchemas {
    pub account: AccountSchema,
    pub post: PostSchema,
}

#[derive(Clone)]
pub struct SearchIndex {
    pub indicies: SearchIndicies,
    pub schemas: SearchSchemas,
}

impl SearchIndex {
    pub fn prepare(config: &Configuration) -> tantivy::Result<Self> {
        let search_schemas = SearchSchemas::default();

        let account_index_dir = config.index_dir_path.join("account");
        let post_index_dir = config.index_dir_path.join("post");

        if !account_index_dir.exists() {
            fs::create_dir_all(&account_index_dir)?;
        }
        if !post_index_dir.exists() {
            fs::create_dir_all(&post_index_dir)?;
        }

        let account_directory = MmapDirectory::open(account_index_dir)?;
        let account_index = Index::open_or_create(
            account_directory,
            search_schemas.account.tantivy_schema.clone(),
        )?;

        let post_directory = MmapDirectory::open(post_index_dir)?;
        let post_index =
            Index::open_or_create(post_directory, search_schemas.post.tantivy_schema.clone())?;

        Ok(Self {
            indicies: SearchIndicies {
                account: account_index,
                post: post_index,
            },
            schemas: search_schemas,
        })
    }
}
