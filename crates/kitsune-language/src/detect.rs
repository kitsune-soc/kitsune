use isolang::Language;

#[derive(Clone, Copy, Debug, Default)]
pub enum DetectionBackend {
    #[default]
    Dummy,
    Whatlang,
    Whichlang,
}

/// Get the ISO code of the specified text
///
/// If the language couldn't get detected reliably, it defaults to english
#[must_use]
pub fn detect_language(backend: DetectionBackend, text: &str) -> Language {
    match backend {
        DetectionBackend::Dummy => Language::Eng,
        DetectionBackend::Whatlang => whatlang::detect(text)
            .and_then(|info| info.is_reliable().then_some(info.lang()))
            .map_or(Language::Eng, crate::map::whatlang_to_isolang),
        DetectionBackend::Whichlang => {
            crate::map::whichlang_to_isolang(whichlang::detect_language(text))
        }
    }
}

#[cfg(test)]
mod test {
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
