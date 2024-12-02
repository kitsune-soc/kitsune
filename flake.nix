{
  inputs = {
    devenv = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:cachix/devenv";
    };

    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    rust-overlay = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:oxalica/rust-overlay";
    };

    crane.url = "github:ipetkov/crane";
    pnpm2nix = {
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
      url = "github:aumetra/pnpm2nix-nzbr";
    };

    # The premise is this is the "default" and if you want to do a debug build,
    # pass it in as an arg.
    # like so `nix build --override-input debugBuild github:boolean-option/true`
    debugBuild.url = "github:boolean-option/false/d06b4794a134686c70a1325df88a6e6768c6b212";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs =
    { self
    , devenv
    , flake-utils
    , nixpkgs
    , rust-overlay
    , crane
    , pnpm2nix
    , ...
    }@inputs:
    (
      flake-utils.lib.eachDefaultSystem
        (
          system:
          let
            features = "--all-features";
            overlays = [ (import rust-overlay) ];
            pkgs = import nixpkgs { inherit overlays system; };
            stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;
            rustPlatform = pkgs.makeRustPlatform {
              cargo = pkgs.rust-bin.stable.latest.minimal;
              rustc = pkgs.rust-bin.stable.latest.minimal;
              inherit stdenv;
            };

            craneLib = (crane.mkLib pkgs).overrideToolchain pkgs.rust-bin.stable.latest.minimal;
            buildInputs = with pkgs; [
            ];

            nativeBuildInputs = with pkgs; [
            ];

            src = pkgs.lib.cleanSourceWith {
              src = pkgs.lib.cleanSource ./.;
              filter =
                name: type:
                let
                  baseName = baseNameOf (toString name);
                in
                  !(baseName == "flake.lock" || pkgs.lib.hasSuffix ".nix" baseName);
            };

            commonArgs =
              let
                excludedPkgs = [ "example-mrf" "http-client-test" ];
                buildExcludeParam = pkgs.lib.strings.concatMapStringsSep " " (pkgName: "--exclude ${pkgName}");
                excludeParam = buildExcludeParam excludedPkgs;
              in
              {
                inherit
                  src
                  stdenv
                  buildInputs
                  nativeBuildInputs
                  ;

                strictDeps = true;

                meta = {
                  description = "ActivityPub-federated microblogging";
                  homepage = "https://joinkitsune.org";
                };

                NIX_OUTPATH_USED_AS_RANDOM_SEED = "aaaaaaaaaa";
                CARGO_PROFILE = "dist";
                cargoExtraArgs = "--locked ${features} --workspace ${excludeParam}";
              }
              // (pkgs.lib.optionalAttrs inputs.debugBuild.value {
                # do a debug build, as `dev` is the default debug profile
                CARGO_PROFILE = "dev";
              });

            cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
            version = cargoToml.workspace.package.version;

            cargoArtifacts = craneLib.buildDepsOnly (
              commonArgs
              // {
                pname = "kitsune-workspace";
                src = craneLib.cleanCargoSource src;
                doCheck = false;
              }
            );
          in
          {
            formatter = pkgs.nixpkgs-fmt;
            packages = rec {
              default = main;

              devenv-up = self.devShells.${system}.default.config.procfileScript;

              cli = craneLib.buildPackage (
                commonArgs
                // {
                  pname = "kitsune-cli";
                  cargoExtraArgs = commonArgs.cargoExtraArgs + " --bin kitsune-cli";
                  inherit cargoArtifacts;
                  doCheck = false;
                }
              );

              cli-docker = pkgs.dockerTools.buildLayeredImage {
                name = "kitsune-cli";
                tag = "latest";
                contents = [ pkgs.dockerTools.caCertificates cli ];
                config.Cmd = [ "${cli}/bin/kitsune-cli" ];
              };

              job-runner = craneLib.buildPackage (
                commonArgs
                // {
                  pname = "kitsune-job-runner";
                  cargoExtraArgs = commonArgs.cargoExtraArgs + " --bin kitsune-job-runner";
                  inherit cargoArtifacts;
                  doCheck = false;
                }
              );

              job-runner-docker = pkgs.dockerTools.buildLayeredImage {
                name = "kitsune-job-runner";
                tag = "latest";
                contents = [ pkgs.dockerTools.caCertificates job-runner ];
                config.Cmd = [ "${job-runner}/bin/kitsune-job-runner" ];
              };

              mrf-tool = craneLib.buildPackage (
                commonArgs
                // {
                  pname = "mrf-tool";
                  cargoExtraArgs = commonArgs.cargoExtraArgs + " --bin mrf-tool";
                  inherit cargoArtifacts;
                  doCheck = false;
                }
              );

              mrf-tool-docker = pkgs.dockerTools.buildLayeredImage {
                name = "mrf-tool";
                tag = "latest";
                contents = [ mrf-tool ];
                config.Cmd = [ "${mrf-tool}/bin/mrf-tool" ];
              };

              main = craneLib.buildPackage (
                commonArgs
                // {
                  pname = "kitsune";
                  cargoExtraArgs = commonArgs.cargoExtraArgs + " --bin kitsune";
                  inherit cargoArtifacts;
                  doCheck = false;
                }
              );

              main-docker = pkgs.dockerTools.buildLayeredImage {
                name = "kitsune";
                tag = "latest";
                contents = [ pkgs.dockerTools.caCertificates main ];
                config.Cmd = [ "${main}/bin/kitsune" ];
              };

              frontend = pnpm2nix.packages.${system}.mkPnpmPackage {
                inherit src;
                distDir = "kitsune-fe/build";
                installInPlace = true;
                packageJSON = "${src}/kitsune-fe/package.json";
                script = "-C kitsune-fe build";
              };

              website = pnpm2nix.packages.${system}.mkPnpmPackage {
                inherit src;
                distDir = "website/dist";
                installInPlace = true;
                packageJSON = "${src}/website/package.json";
                script = "-C website build";
              };
            };

            devShells = rec {
              default = backend;

              backend = devenv.lib.mkShell {
                inherit pkgs inputs;

                modules = [
                  (
                    { pkgs, ... }:
                    {
                      packages =
                        with pkgs;
                        [
                          cargo-insta
                          diesel-cli
                          rust-bin.stable.latest.default
                        ]
                        ++ buildInputs
                        ++ nativeBuildInputs;

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
                        redis = {
                          package = pkgs.valkey;
                          enable = true;
                        };
                      };
                    }
                  )
                ];
              };

              frontend = pkgs.mkShell {
                buildInputs = with pkgs; [
                  nodejs
                  nodePackages.svelte-language-server
                  nodePackages.typescript-language-server
                  pnpm
                ];
              };
            };
          }
        )
      // {
        overlays = rec {
          default = kitsune;
          kitsune = (import ./overlay.nix self);
        };

        nixosModules = rec {
          default = kitsune;
          kitsune = (import ./module.nix);
        };
      }
    )
    // {
      nixci.default = {
        debug = {
          dir = ".";
          overrideInputs.debugBuild = "github:boolean-option/true/6ecb49143ca31b140a5273f1575746ba93c3f698";
        };
      };
    };
}
