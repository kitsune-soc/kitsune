fn main() {
    tonic_build::configure()
        .build_client(false)
        .compile(&["proto/index.proto", "proto/search.proto"], &["proto"])
        .unwrap();
}
