//!
//! Module containing enum definitions for the tsvector columns (implementing the [`Iden`] trait for usage with SeaORM)
//!

use sea_orm::{sea_query, Iden};

/// `accounts` table tsvector columns
#[derive(Clone, Copy, Iden)]

pub enum Accounts {
    /// Display Name
    #[iden = "display_name_tsvector"]
    DisplayName,

    /// Note
    #[iden = "note_tsvector"]
    Note,

    /// Username
    #[iden = "username_tsvector"]
    Username,
}

/// `posts` table tsvector columns
#[derive(Clone, Copy, Iden)]
pub enum Posts {
    /// Subject
    #[iden = "subject_tsvector"]
    Subject,

    /// Content
    #[iden = "content_tsvector"]
    Content,
}
