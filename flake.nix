{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, flake-utils, nixpkgs, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit overlays system;
        };
        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.rust-bin.stable.latest.minimal;
          rustc = pkgs.rust-bin.stable.latest.minimal;
        };
        baseDependencies = with pkgs; [
          openssl
          pkg-config
          protobuf
          sqlite
          zlib
        ];
        cargoConfig = builtins.fromTOML (builtins.readFile ./.cargo/config.toml);  # TODO: Set the target CPU conditionally
        cargoToml = builtins.fromTOML (builtins.readFile ./kitsune/Cargo.toml);
        basePackage = {
          inherit (cargoToml.package) version;

          meta = {
            description = "ActivityPub-federated microblogging";
            homepage = "https://joinkitsune.org";
          };

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          src = pkgs.lib.cleanSource ./.;
          nativeBuildInputs = baseDependencies;

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig"; # Not sure why this is broken but it is
          RUSTFLAGS = builtins.concatStringsSep " " cargoConfig.build.rustflags; # Oh god help.

          checkFlags = [
            # Depend on creating an HTTP client and that reads from the systems truststore
            # Because nix is fully isolated, these types of tests fail
            #
            # Some (most?) of these also depend on the network? Not good??
            "--skip=activitypub::fetcher::test::federation_allow"
            "--skip=activitypub::fetcher::test::federation_deny"
            "--skip=activitypub::fetcher::test::fetch_actor"
            "--skip=activitypub::fetcher::test::fetch_note"
            "--skip=resolve::post::test::parse_mentions"
            "--skip=webfinger::test::fetch_qarnax_ap_id"
            "--skip=basic_request"
            "--skip=json_request"
          ];
        };
      in
      {
        packages = rec {
          default = main;
          cli = rustPlatform.buildRustPackage (basePackage // {
            pname = "kitsune-cli";
            cargoBuildFlags = "-p kitsune-cli";
          });
          main = rustPlatform.buildRustPackage (basePackage // {
            pname = "kitsune";
            buildFeatures = [ "meilisearch" ];
            cargoBuildFlags = "-p kitsune";
          });
          search = rustPlatform.buildRustPackage (basePackage // {
            pname = "kitsune-search";
            cargoBuildFlags = "-p kitsune-search";
          });
        };
        devShells = rec {
          default = backend;
          backend = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo-insta
              dhall
              diesel-cli
              redis
              rust-bin.stable.latest.default
            ]
            ++
            baseDependencies;
          };
          frontend = pkgs.mkShell {
            buildInputs = with pkgs; [
              nodejs
              yarn
            ];
          };
        };
      }
    );
}
