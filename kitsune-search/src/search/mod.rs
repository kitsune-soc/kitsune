//!
//! Search index management
//!

use self::schema::{AccountSchema, PostSchema};
use crate::config::Configuration;
use std::fs;
use tantivy::{directory::MmapDirectory, Index};

pub mod schema;

/// Collection of the managed search indices
#[derive(Clone)]
pub struct SearchIndices {
    /// Account search index
    pub account: Index,

    /// Post search index
    pub post: Index,
}

/// Collections of the schemas of the managed search indices
#[derive(Clone, Default)]
pub struct SearchSchemas {
    /// Account search index schema
    pub account: AccountSchema,

    /// Post search index schema
    pub post: PostSchema,
}

/// The overarching search index
///
/// Contains all the managed schemas and indices
#[derive(Clone)]
pub struct SearchIndex {
    /// Managed indices
    pub indices: SearchIndices,

    /// Managed schemas
    pub schemas: SearchSchemas,
}

impl SearchIndex {
    /// Create or open a search index
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
            indices: SearchIndices {
                account: account_index,
                post: post_index,
            },
            schemas: search_schemas,
        })
    }
}
