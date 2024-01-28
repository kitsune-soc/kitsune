use isolang::Language;
use kitsune_config::language_detection::{self, DetectionBackend};

/// Get the ISO code of the specified text
///
/// If the language couldn't get detected reliably, it defaults to english
#[must_use]
pub fn detect_language(config: language_detection::Configuration, text: &str) -> Language {
    match config.backend {
        DetectionBackend::None => config.default_language,
        DetectionBackend::Whatlang => whatlang::detect(text)
            .and_then(|info| info.is_reliable().then_some(info.lang()))
            .map_or(config.default_language, crate::map::whatlang_to_isolang),
        DetectionBackend::Whichlang => {
            // `whichlang` currently panics if it encounters an empty string
            if text.is_empty() {
                return config.default_language;
            }

            crate::map::whichlang_to_isolang(whichlang::detect_language(text))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::detect_language;
    use isolang::Language;
    use kitsune_config::language_detection::{self, DetectionBackend};

    #[test]
    fn empty_no_panic_whichlang() {
        let empty = "";
        let spaces = " ";

        let config = language_detection::Configuration {
            backend: DetectionBackend::Whichlang,
            default_language: Language::Eng,
        };

        let _ = detect_language(config, empty);
        let _ = detect_language(config, spaces);
    }

    #[test]
    fn supported_includes_detection_languages() {
        use crate::{
            map::{whatlang_to_isolang, whichlang_to_isolang},
            supported_languages,
        };

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
