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
            rustToolchain = pkgs.rust-bin.stable.latest.minimal;

            rustPlatform = pkgs.makeRustPlatform {
              cargo = rustToolchain;
              rustc = rustToolchain;
              inherit stdenv;
            };

            craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

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
                inherit src stdenv;

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
            packages = {
              devenv-up = self.devShells.${system}.default.config.procfileScript;
            } // (import ./nix/packages.nix) { inherit commonArgs craneLib pkgs pnpm2nix; };

            devShells = (import ./nix/devshells.nix) { inherit devenv pkgs inputs; };
          }
        )
      // {
        overlays = rec {
          default = kitsune;
          kitsune = (import ./nix/overlay.nix self);
        };

        nixosModules = rec {
          default = kitsune;
          kitsune = (import ./nix/module.nix);
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
