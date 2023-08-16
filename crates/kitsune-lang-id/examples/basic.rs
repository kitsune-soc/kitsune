use kitsune_lang_id::DetectionBackend;

fn main() {
    kitsune_lang_id::set_backend(DetectionBackend::Whichlang);

    let detected = kitsune_lang_id::detect_language("das ist schon eine coole library..");
    println!("Detected language: {detected}");
}
