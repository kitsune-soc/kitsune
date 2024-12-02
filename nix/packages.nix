{ commonArgs, craneLib, pkgs, pnpm2nix }:
{
  default = main;
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
}
