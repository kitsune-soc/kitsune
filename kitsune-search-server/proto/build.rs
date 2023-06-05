fn main() {
    #[cfg(feature = "vendored-protoc")]
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .type_attribute(".", "#[derive(::serde::Serialize)]")
        .compile(
            &[
                "../../proto/search/common.proto",
                "../../proto/search/index.proto",
                "../../proto/search/search.proto",
            ],
            &["../../proto/search"],
        )
        .unwrap();
}
