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
        baseDependencies = with pkgs; [
          openssl
          protobuf
          sqlite
          zlib
        ];
        basePackage = {
          version = "0.0.1-pre.0";
          meta = {
            description = "ActivityPub-federated microblogging";
            homepage = "https://joinkitsune.org";
          };

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          src = pkgs.lib.cleanSource ./.;
          buildInputs = baseDependencies;
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
