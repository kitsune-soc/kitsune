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

          src = ./.;
        };
      in
      {
        packages = {
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
