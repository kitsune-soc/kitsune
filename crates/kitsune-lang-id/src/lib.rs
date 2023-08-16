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
    #[cfg(feature = "lingua")]
    return ArcSwap::new(DetectionBackend::Lingua.into());

    #[cfg(feature = "whatlang")]
    return ArcSwap::new(DetectionBackend::Whatlang.into());

    #[cfg(feature = "whichlang")]
    return ArcSwap::new(DetectionBackend::Whichlang.into());

    ArcSwap::new(DetectionBackend::Dummy.into())
});

#[cfg(feature = "lingua")]
static LINGUA_DETECTOR: Lazy<lingua::LanguageDetector> = Lazy::new(|| {
    lingua::LanguageDetectorBuilder::from_all_languages()
        .with_preloaded_language_models()
        .build()
});

pub enum DetectionBackend {
    Dummy,
    #[cfg(feature = "lingua")]
    Lingua,
    #[cfg(feature = "whatlang")]
    Whatlang,
    #[cfg(feature = "whichlang")]
    Whichlang,
}

pub fn set_backend(backend: DetectionBackend) {
    DETECTION_BACKEND.store(backend.into());
}

#[inline]
pub fn supported_languages() -> impl Iterator<Item = Language> {
    // Manual override for languages that are either explicitly requested to be supported, or are supported by the detection backend
    let manually_added_languages = [
        Language::Ast,
        Language::Ckb,
        Language::Cmn,
        Language::Cnr,
        Language::Jbo,
        Language::Kab,
        Language::Kmr,
        Language::Ldn,
        Language::Lfn,
        Language::Pes,
        Language::Sco,
        Language::Sma,
        Language::Smj,
        Language::Szl,
        Language::Tok,
        Language::Zba,
        Language::Zgh,
    ];

    isolang::languages()
        .filter(|lang| lang.to_639_1().is_some())
        .chain(manually_added_languages)
}

/// Get the ISO code of the specified text
///
/// If the language couldn't get detected reliably, it defaults to english
#[must_use]
#[allow(unused_variables)] // In case we don't have any detectors compiled
pub fn detect_language(text: &str) -> Language {
    match **DETECTION_BACKEND.load() {
        DetectionBackend::Dummy => Language::Eng,
        #[cfg(feature = "lingua")]
        DetectionBackend::Lingua => LINGUA_DETECTOR
            .detect_language_of(text)
            .map_or(Language::Eng, self::map::lingua_to_isolang),
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

#[cfg(test)]
mod test {
    use crate::supported_languages;
    use isolang::Language;
    use std::collections::HashSet;

    #[test]
    fn no_duplicate_languages() {
        let language_hashset = supported_languages().collect::<HashSet<Language>>();
        assert_eq!(language_hashset.len(), supported_languages().count());
    }

    #[cfg(all(feature = "lingua", feature = "whatlang", feature = "whichlang"))]
    #[test]
    fn supported_includes_detection_languages() {
        use crate::map::{lingua_to_isolang, whatlang_to_isolang, whichlang_to_isolang};

        for lang in lingua::Language::all() {
            assert!(
                supported_languages()
                    .any(|supported_lang| supported_lang == lingua_to_isolang(lang)),
                "Unsupported language {lang:?}"
            );
        }

        for lang in whatlang::Lang::all() {
            assert!(
                supported_languages()
                    .any(|supported_lang| supported_lang == whatlang_to_isolang(*lang)),
                "Unsupported language {lang:?}"
            );
        }

        for lang in whichlang::LANGUAGES {
            assert!(
                supported_languages()
                    .any(|supported_lang| supported_lang == whichlang_to_isolang(lang)),
                "Unsupported language {lang:?}"
            );
        }
    }
}
