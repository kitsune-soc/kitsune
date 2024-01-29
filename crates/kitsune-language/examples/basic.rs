use isolang::Language;
use kitsune_config::language_detection::{self, DetectionBackend};

fn main() {
    let detected = kitsune_language::detect_language(
        language_detection::Configuration {
            backend: DetectionBackend::Whichlang,
            default_language: Language::Eng,
        },
        "das ist schon eine coole library..",
    );
    println!("Detected language: {detected}");
}
