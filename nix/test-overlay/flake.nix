{
  inputs = {
    kitsune-overlay.url = "./../..";
    kitsune-overlay.inputs.debugBuild.follows = "debugBuild";
    nixpkgs.follows = "kitsune-overlay/nixpkgs";
    flake-utils.follows = "kitsune-overlay/flake-utils";
    debugBuild.url = "github:boolean-option/true/6ecb49143ca31b140a5273f1575746ba93c3f698";
  };
  outputs = { self, flake-utils, nixpkgs, kitsune-overlay, ... } @ inputs:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ kitsune-overlay.overlays.default ];
          pkgs = import nixpkgs {
            inherit overlays system;
          };
        in
        {
          formatter = pkgs.nixpkgs-fmt;
          packages = rec {
            default = kitsune;
            inherit (pkgs) kitsune;
            inherit (pkgs) kitsune-cli;
          };
        }
      );
}
