self: final: prev:
let
  packages = self.packages.${prev.stdenv.targetPlatform.system};
in
{
  kitsune = prev.stdenv.mkDerivation {
    inherit (packages.main) name version meta src;

    installPhase = ''
      mkdir -p $out
      cp -R ${packages.main}/bin $out
      cp -R ${packages.main.src}/kitsune/assets $out/public
      cp -R ${packages.frontend}/dist $out/kitsune-fe
    '';
  };

  kitsune-cli = packages.cli;
}
