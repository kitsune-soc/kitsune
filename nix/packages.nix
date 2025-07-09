{ crane, debugBuild, pkgs, mkPnpmPackage }:
let
  features = "--all-features";

  stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;
  rustToolchain = pkgs.rust-bin.stable.latest.minimal;

  rustPlatform = pkgs.makeRustPlatform {
    cargo = rustToolchain;
    rustc = rustToolchain;
    inherit stdenv;
  };

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

  cargoToml = builtins.fromTOML (builtins.readFile ./../Cargo.toml);
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
rec {
  default = main;

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

  frontend = mkPnpmPackage {
    inherit src;
    distDir = "kitsune-fe/build";
    installInPlace = true;
    packageJSON = "${src}/kitsune-fe/package.json";
    script = "-C kitsune-fe build";
  };

  website = mkPnpmPackage {
    inherit src;
    distDir = "website/dist";
    installInPlace = true;
    packageJSON = "${src}/website/package.json";
    script = "-C website build";
  };
}
