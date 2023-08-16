use isolang::Language;

#[cfg(feature = "lingua")]
static LINGUA_DETECTOR: once_cell::sync::Lazy<lingua::LanguageDetector> =
    once_cell::sync::Lazy::new(|| {
        lingua::LanguageDetectorBuilder::from_all_languages()
            .with_preloaded_language_models()
            .build()
    });

#[derive(Clone, Copy, Debug)]
pub enum DetectionBackend {
    Dummy,
    #[cfg(feature = "lingua")]
    Lingua,
    #[cfg(feature = "whatlang")]
    Whatlang,
    #[cfg(feature = "whichlang")]
    Whichlang,
}

impl Default for DetectionBackend {
    #[allow(unreachable_code)]
    fn default() -> Self {
        #[cfg(feature = "lingua")]
        return Self::Lingua;

        #[cfg(feature = "whatlang")]
        return Self::Whatlang;

        #[cfg(feature = "whichlang")]
        return Self::Whichlang;

        Self::Dummy
    }
}

/// Get the ISO code of the specified text
///
/// If the language couldn't get detected reliably, it defaults to english
#[must_use]
#[allow(unused_variables)] // In case we don't have any detectors compiled
pub fn detect_language(backend: DetectionBackend, text: &str) -> Language {
    match backend {
        DetectionBackend::Dummy => Language::Eng,
        #[cfg(feature = "lingua")]
        DetectionBackend::Lingua => LINGUA_DETECTOR
            .detect_language_of(text)
            .map_or(Language::Eng, crate::map::lingua_to_isolang),
        #[cfg(feature = "whatlang")]
        DetectionBackend::Whatlang => whatlang::detect(text)
            .and_then(|info| info.is_reliable().then_some(info.lang()))
            .map_or(Language::Eng, crate::map::whatlang_to_isolang),
        #[cfg(feature = "whichlang")]
        DetectionBackend::Whichlang => {
            crate::map::whichlang_to_isolang(whichlang::detect_language(text))
        }
    }
}

#[cfg(test)]
mod test {
    #[cfg(all(feature = "lingua", feature = "whatlang", feature = "whichlang"))]
    #[test]
    fn supported_includes_detection_languages() {
        use crate::{
            map::{lingua_to_isolang, whatlang_to_isolang, whichlang_to_isolang},
            supported_languages,
        };

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
