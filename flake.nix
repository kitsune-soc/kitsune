{
  inputs = {
    devenv = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:cachix/devenv";
    };

    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    rust-overlay = {
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
      url = "github:oxalica/rust-overlay";
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # The premise is this is the "default" and if you want to do a debug build,
    # pass it in as an arg.
    # like so `nix build --override-input debugBuild github:boolean-option/true`
    debugBuild.url = "github:boolean-option/false/d06b4794a134686c70a1325df88a6e6768c6b212";
  };
  outputs = { self, devenv, flake-utils, nixpkgs, rust-overlay, crane, ... } @ inputs:
    (flake-utils.lib.eachDefaultSystem
      (system:
        let
          features = "--all-features";
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit overlays system;
          };
          stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = pkgs.rust-bin.stable.latest.minimal;
            rustc = pkgs.rust-bin.stable.latest.minimal;
            inherit stdenv;
          };

          craneLib = (crane.mkLib pkgs).overrideToolchain pkgs.rust-bin.stable.latest.minimal;
          buildInputs = with pkgs; [
            openssl
            sqlite
            zlib
          ];

          nativeBuildInputs = with pkgs; [
            protobuf
            pkg-config
            rustPlatform.bindgenHook
          ];

          src = pkgs.lib.cleanSourceWith {
            src = pkgs.lib.cleanSource ./.;
            filter = name: type:
              let baseName = baseNameOf (toString name);
              in !(baseName == "flake.lock" || pkgs.lib.hasSuffix ".nix" baseName);
          };

          commonArgs = {
            inherit src stdenv buildInputs nativeBuildInputs;

            strictDeps = true;

            meta = {
              description = "ActivityPub-federated microblogging";
              homepage = "https://joinkitsune.org";
            };

            OPENSSL_NO_VENDOR = 1;
            NIX_OUTPATH_USED_AS_RANDOM_SEED = "aaaaaaaaaa";
            cargoExtraArgs = "--locked ${features}";
          } // (pkgs.lib.optionalAttrs inputs.debugBuild.value {
            # do a debug build, as `dev` is the default debug profile
            CARGO_PROFILE = "dev";
          });

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          version = cargoToml.workspace.package.version;

          cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
            pname = "kitsune-workspace";
            src = craneLib.cleanCargoSource src;
          });
        in
        {
          formatter = pkgs.nixpkgs-fmt;
          packages = rec {
            default = main;

            cli = craneLib.buildPackage (commonArgs // {
              pname = "kitsune-cli";
              cargoExtraArgs = commonArgs.cargoExtraArgs + " --bin kitsune-cli";
              inherit cargoArtifacts;
              doCheck = false;
            });

            main = craneLib.buildPackage (commonArgs // rec {
              pname = "kitsune";
              cargoExtraArgs = commonArgs.cargoExtraArgs + " --bin kitsune";
              inherit cargoArtifacts;
              doCheck = false;
            });

            frontend = pkgs.mkYarnPackage {
              inherit version;
              packageJSON = "${src}/kitsune-fe/package.json";
              yarnLock = "${src}/kitsune-fe/yarn.lock";
              src = "${src}/kitsune-fe";

              buildPhase = ''
                export HOME=$(mktemp -d)
                yarn --offline build
              '';

              installPhase = ''
                mkdir -p $out
                cp -R deps/kitsune-fe/dist $out
              '';

              distPhase = "true";
            };
          };

          devShells = rec {
            default = backend;

            backend = devenv.lib.mkShell {
              inherit pkgs inputs;

              modules = [
                ({ pkgs, ... }: {
                  packages = with pkgs; [
                    cargo-insta
                    diesel-cli
                    rust-bin.stable.latest.default
                  ]
                  ++
                  buildInputs ++ nativeBuildInputs;

                  enterShell = ''
                    export PG_HOST=127.0.0.1
                    export PG_PORT=5432
                    [ -z "$DATABASE_URL" ] && export DATABASE_URL=postgres://$USER@$PG_HOST:$PG_PORT/$USER

                    export REDIS_PORT=6379
                    [ -z "$REDIS_URL" ] && export REDIS_URL="redis://127.0.0.1:$REDIS_PORT"
                  '';

                  services = {
                    postgres = {
                      enable = true;
                      listen_addresses = "127.0.0.1";
                    };
                    redis.enable = true;
                  };
                })
              ];
            };

            frontend = pkgs.mkShell {
              buildInputs = with pkgs; [
                nodejs
                yarn
              ];
            };
          };
        }
      ) // {
      overlays = rec {
        default = kitsune;
        kitsune = (import ./overlay.nix self);
      };

      nixosModules = rec {
        default = kitsune;
        kitsune = (import ./module.nix);
      };
    }) // {
      nixci.default = {
        debug = {
          dir = ".";
          overrideInputs.debugBuild = "github:boolean-option/true/6ecb49143ca31b140a5273f1575746ba93c3f698";
        };
      };
    };
}
