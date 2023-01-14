fn main() {
    tonic_build::configure()
        .build_client(false)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["proto/index.proto", "proto/search.proto"], &["proto"])
        .unwrap();
}
