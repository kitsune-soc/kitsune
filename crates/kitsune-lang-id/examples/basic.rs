use kitsune_lang_id::DetectionBackend;

fn main() {
    let detected = kitsune_lang_id::detect_language(
        DetectionBackend::default(),
        "das ist schon eine coole library..",
    );
    println!("Detected language: {detected}");
}
