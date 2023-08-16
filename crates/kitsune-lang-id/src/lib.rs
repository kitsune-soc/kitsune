#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use arc_swap::ArcSwap;
use once_cell::sync::Lazy;

mod map;
mod pg_enum;
mod regconfig;

pub use self::{pg_enum::generate_postgres_enum, regconfig::generate_regconfig_function};
pub use isolang::Language;

#[allow(unreachable_code)]
static DETECTION_BACKEND: Lazy<ArcSwap<DetectionBackend>> = Lazy::new(|| {
    #[cfg(feature = "whatlang")]
    return ArcSwap::new(DetectionBackend::Whatlang.into());

    #[cfg(feature = "whichlang")]
    return ArcSwap::new(DetectionBackend::Whichlang.into());

    ArcSwap::new(DetectionBackend::Dummy.into())
});

pub enum DetectionBackend {
    Dummy,
    #[cfg(feature = "whatlang")]
    Whatlang,
    #[cfg(feature = "whichlang")]
    Whichlang,
}

pub fn set_backend(backend: DetectionBackend) {
    DETECTION_BACKEND.store(backend.into());
}

#[inline]
fn supported_languages() -> impl Iterator<Item = isolang::Language> {
    isolang::languages().filter(|lang| lang.to_639_1().is_some())
}

/// Get the ISO code of the specified text
///
/// If the language couldn't get detected reliably, it defaults to english
#[must_use]
#[allow(unused_variables)] // In case we don't have any detectors compiled
pub fn detect_language(text: &str) -> Language {
    match **DETECTION_BACKEND.load() {
        DetectionBackend::Dummy => Language::Eng,
        #[cfg(feature = "whatlang")]
        DetectionBackend::Whatlang => whatlang::detect(text)
            .and_then(|info| info.is_reliable().then_some(info.lang()))
            .map_or(Language::Eng, self::map::whatlang_to_isolang),
        #[cfg(feature = "whichlang")]
        DetectionBackend::Whichlang => {
            self::map::whichlang_to_isolang(whichlang::detect_language(text))
        }
    }
}
