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
            in
            {
                devShells.default = pkgs.mkShell {
                    buildInputs = with pkgs; [
                        cargo-insta
                        dhall
                        nodejs
                        openssl_1_1
                        #postgresql_15
                        protobuf
                        redis
                        rust-bin.stable.latest.default
                        sqlite
                        yarn
                    ];
                };
            }
        );
}
