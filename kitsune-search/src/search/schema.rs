//!
//! Schemas managed by the service
//!

use tantivy::{
    query::{BooleanQuery, FuzzyTermQuery, Query},
    schema::{Field, Schema, INDEXED, STORED, STRING, TEXT},
    Term,
};

/// Trait for preparing a tantivy query for some schema
pub trait PrepareQuery {
    /// Type of the returned query
    type Query: Query;

    /// Prepare a tantivy query
    fn prepare_query(&self, query: &str, levenshtein_distance: u8) -> Self::Query;
}

/// Account search schema
#[derive(Clone)]
pub struct AccountSchema {
    /// ID field (contains a UUID)
    pub id: Field,

    /// Display name field (might be empty)
    pub display_name: Field,

    /// Username field
    pub username: Field,

    /// Description (or bio, etc.) field (might be empty)
    pub description: Field,

    /// The underlying tantivy schema with the above defined fields
    pub tantivy_schema: Schema,
}

impl Default for AccountSchema {
    fn default() -> Self {
        let mut builder = Schema::builder();
        let id = builder.add_bytes_field("id", INDEXED | STORED);
        let display_name = builder.add_text_field("display_name", TEXT);
        let username = builder.add_text_field("username", STRING);
        let description = builder.add_text_field("description", TEXT);
        let tantivy_schema = builder.build();

        Self {
            id,
            display_name,
            username,
            description,
            tantivy_schema,
        }
    }
}

impl PrepareQuery for AccountSchema {
    type Query = BooleanQuery;

    fn prepare_query(&self, query: &str, levenshtein_distance: u8) -> Self::Query {
        let queries: Vec<Box<dyn Query + 'static>> = vec![
            Box::new(FuzzyTermQuery::new(
                Term::from_field_text(self.display_name, query),
                levenshtein_distance,
                true,
            )),
            Box::new(FuzzyTermQuery::new(
                Term::from_field_text(self.username, query),
                levenshtein_distance,
                true,
            )),
            Box::new(FuzzyTermQuery::new(
                Term::from_field_text(self.description, query),
                levenshtein_distance,
                true,
            )),
        ];

        BooleanQuery::union(queries)
    }
}

/// Post search schema
#[derive(Clone)]
pub struct PostSchema {
    /// ID field (contains a UUID)
    pub id: Field,

    /// Subject field (might be empty)
    pub subject: Field,

    /// Content field
    pub content: Field,

    /// The underlying tantivy schema with the above defined fields
    pub tantivy_schema: Schema,
}

impl Default for PostSchema {
    fn default() -> Self {
        let mut builder = Schema::builder();
        let id = builder.add_bytes_field("id", INDEXED | STORED);
        let subject = builder.add_text_field("subject", TEXT);
        let content = builder.add_text_field("content", TEXT);
        let tantivy_schema = builder.build();

        Self {
            id,
            subject,
            content,
            tantivy_schema,
        }
    }
}

impl PrepareQuery for PostSchema {
    type Query = BooleanQuery;

    fn prepare_query(&self, query: &str, levenshtein_distance: u8) -> Self::Query {
        let queries: Vec<Box<dyn Query + 'static>> = vec![
            Box::new(FuzzyTermQuery::new(
                Term::from_field_text(self.subject, query),
                levenshtein_distance,
                true,
            )),
            Box::new(FuzzyTermQuery::new(
                Term::from_field_text(self.content, query),
                levenshtein_distance,
                true,
            )),
        ];

        BooleanQuery::union(queries)
    }
}
