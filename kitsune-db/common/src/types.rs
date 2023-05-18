//!
//! Utility types that aren't covered by `sea-query`
//!

use sea_orm::{sea_query, Iden};

/// PostgreSQL-exclusive types
#[derive(Clone, Copy, Iden)]
pub enum PgTypes {
    /// tsvector (text-search vector)
    Tsvector,
}

/// PostgreSQL-exclusive composite-type indices
#[derive(Clone, Copy, Iden)]
pub enum PgCompositeIndex {
    /// GIN (Generalized Inverted Index)
    #[iden = "GIN"]
    Gin,

    /// RUM index
    ///
    /// RUM is an improved version of GIN that improves it speed at the cost of disk space usage
    #[iden = "RUM"]
    Rum,
}
