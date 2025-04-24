{ crane, debugBuild, pkgs }:
let
  rustData = (import ./build-rust.nix) { inherit crane debugBuild pkgs; };
in
{
  clippy = rustData.craneLib.cargoClippy (rustData.commonArgs // {
    pname = "kitsune-clippy";
    cargoArtifacts = rustData.cargoArtifacts;
    cargoClippyExtraArgs = "--all-targets -- --deny warnings";
  });

  fmt = rustData.craneLib.cargoFmt {
    pname = "kitsune-fmt";
    src = rustData.src;
  };

  toml-fmt = rustData.craneLib.taploFmt {
    pname = "kitsune-toml-fmt";
    src = pkgs.lib.sources.sourceFilesBySuffices rustData.src [ ".toml" ];
  };
}
