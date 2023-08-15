#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

#[cfg(all(feature = "whatlang", feature = "whichlang"))]
compile_error!("Only enable one of the language detector features");

#[cfg(not(any(feature = "whatlang", feature = "whichlang")))]
compile_error!("Enable one of the language detector features");

mod map;
mod pg_enum;
mod regconfig;

pub use self::{pg_enum::generate_postgres_enum, regconfig::generate_regconfig_function};
pub use isolang::Language;

#[inline]
fn supported_languages() -> impl Iterator<Item = isolang::Language> {
    isolang::languages().filter(|lang| lang.to_639_1().is_some())
}

/// Get the ISO code of the specified text
///
/// If the language couldn't get detected reliably, it defaults to english
#[must_use]
pub fn get_iso_code(text: &str) -> Language {
    #[cfg(feature = "whatlang")]
    {
        whatlang::detect(text)
            .and_then(|info| info.is_reliable().then_some(info.lang()))
            .map_or(Language::Eng, self::map::whatlang_to_isolang)
    }

    #[cfg(feature = "whichlang")]
    self::map::whichlang_to_isolang(whichlang::detect_language(text))
}
