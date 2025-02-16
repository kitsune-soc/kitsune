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

    crane.url = "github:ipetkov/crane/a5cbb5715849a80707477865f67803528c696d9d"; # ToDo: unpin when this stuff doesn't break anymore.
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
    , crane
    , devenv
    , flake-utils
    , nixpkgs
    , pnpm2nix
    , rust-overlay
    , ...
    }@inputs:
    (
      flake-utils.lib.eachDefaultSystem
        (
          system:
          let
            overlays = [ (import rust-overlay) ];
            pkgs = import nixpkgs { inherit overlays system; };
          in
          {
            formatter = pkgs.nixpkgs-fmt;
            packages = {
              devenv-up = self.devShells.${system}.default.config.procfileScript;
            } // (import ./nix/packages.nix) {
              inherit crane pkgs;

              debugBuild = inputs.debugBuild;
              mkPnpmPackage = pnpm2nix.packages.${system}.mkPnpmPackage;
            };

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
