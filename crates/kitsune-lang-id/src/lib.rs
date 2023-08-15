#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use self::map::whatlang_to_isolang;

mod map;
mod pg_enum;
mod regconfig;

pub use self::{pg_enum::generate_postgres_enum, regconfig::generate_regconfig_function};
pub use isolang::Language;

/// Get the ISO code of the specified text
///
/// If the language couldn't get detected reliably, it defaults to english
#[must_use]
pub fn get_iso_code(text: &str) -> Language {
    whatlang::detect(text)
        .and_then(|info| info.is_reliable().then_some(info.lang()))
        .map_or(Language::Eng, whatlang_to_isolang)
}
