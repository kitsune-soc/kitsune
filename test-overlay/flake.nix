{
  inputs = {
    kitsune-overlay.url = "./..";
    nixpkgs.follows = "kitsune-overlay/nixpkgs";
    flake-utils.follows = "kitsune-overlay/flake-utils";
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
