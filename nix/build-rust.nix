{ crane, debugBuild, pkgs }:
let
  features = "--all-features";

  stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;
  rustToolchain = pkgs.rust-bin.stable.latest.default;

  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
  src = pkgs.lib.cleanSourceWith {
    src = pkgs.lib.cleanSource ./..;
    filter =
      name: type:
      let
        baseName = baseNameOf (toString name);
      in
        !(baseName == "flake.lock" || pkgs.lib.hasSuffix ".nix" baseName);
  };
in
rec {
  inherit craneLib src;

  commonArgs =
    let
      excludedPkgs = [ "example-mrf" "http-client-test" ];
      buildExcludeParam = pkgs.lib.strings.concatMapStringsSep " " (pkgName: "--exclude ${pkgName}");
      excludeParam = buildExcludeParam excludedPkgs;
    in
    {
      inherit src stdenv;

      cargoExtraArgs = "--locked ${features} --workspace ${excludeParam}";
      strictDeps = true;

      CARGO_PROFILE = "dist";
      NIX_OUTPATH_USED_AS_RANDOM_SEED = "aaaaaaaaaa";
    }
    // (pkgs.lib.optionalAttrs debugBuild.value {
      # do a debug build, as `dev` is the default debug profile
      CARGO_PROFILE = "dev";
    });

  cargoArtifacts = craneLib.buildDepsOnly (
    commonArgs
    // {
      pname = "kitsune-workspace";
      doCheck = false;
    }
  );
}
