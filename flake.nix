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
                    cargo = pkgs.rust-bin.stable.latest.default;
                    rustc = pkgs.rust-bin.stable.latest.default;
                };
            in
            {
                packages.cli = rustPlatform.buildRustPackage {
                    pname = "kitsune-cli";
                    version = "0.0.1-pre.0";

                    meta = {
                        description = "ActivityPub-federated microblogging";
                        homepage = "https://joinkitsune.org";
                    };

                    cargoBuildFlags = "-p kitsune-cli";
                    cargoLock = {
                        lockFile = ./Cargo.lock;
                        allowBuiltinFetchGit = true;
                    };

                    src = ./.;
                };
                packages.main = rustPlatform.buildRustPackage {
                    pname = "kitsune";
                    version = "0.0.1-pre.0";

                    meta = {
                        description = "ActivityPub-federated microblogging";
                        homepage = "https://joinkitsune.org";
                    };

                    buildFeatures = [ "meilisearch" ];
                    cargoBuildFlags = "-p kitsune";
                    cargoLock = {
                        lockFile = ./Cargo.lock;
                        allowBuiltinFetchGit = true;
                    };

                    src = ./.;
                };
                packages.search = rustPlatform.buildRustPackage {
                    pname = "kitsune-search";
                    version = "0.0.1-pre.0";

                    meta = {
                        description = "ActivityPub-federated microblogging";
                        homepage = "https://joinkitsune.org";
                    };

                    cargoBuildFlags = "-p kitsune-search";
                    cargoLock = {
                        lockFile = ./Cargo.lock;
                        allowBuiltinFetchGit = true;
                    };

                    src = ./.;
                };
                devShells.default = pkgs.mkShell {
                    buildInputs = with pkgs; [
                        cargo-insta
                        dhall
                        nodejs
                        openssl
                        protobuf
                        redis
                        rust-bin.stable.latest.default
                        sqlite
                        yarn
                        zlib
                    ];
                };
            }
        );
}
