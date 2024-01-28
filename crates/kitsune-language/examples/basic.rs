use kitsune_language::DetectionBackend;

fn main() {
    let detected = kitsune_language::detect_language(
        DetectionBackend::Whichlang,
        "das ist schon eine coole library..",
    );
    println!("Detected language: {detected}");
}
