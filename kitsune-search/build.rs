fn main() {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &[
                "proto/common.proto",
                "proto/index.proto",
                "proto/search.proto",
            ],
            &["proto"],
        )
        .unwrap();
}
