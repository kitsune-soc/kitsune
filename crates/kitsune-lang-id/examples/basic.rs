fn main() {
    #[cfg(feature = "whichlang")]
    kitsune_lang_id::set_backend(kitsune_lang_id::DetectionBackend::Whichlang);

    let detected = kitsune_lang_id::detect_language("das ist schon eine coole library..");
    println!("Detected language: {detected}");
}
