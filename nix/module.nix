{ config, lib, pkgs, ... }:
let
  inherit (lib) types mkEnableOption mkOption;
  inherit (builtins) toJSON;
  cfg = config.services.kitsune;
  format = pkgs.formats.toml { };
  configFile = format.generate "config.toml" cfg.config;

  # based on gist linked in <https://discourse.nixos.org/t/problems-with-types-oneof-and-submodules/15197>
  taggedSubmodule = typeTag: options:
    let
      submodule = types.submodule {
        freeformType = format.type;
        options = options // {
          type = mkOption {
            type = types.enum [ typeTag ];
          };
        };
      };
    in
    submodule // {
      check = v: (submodule.check v) && (v.type == typeTag);
    };
  oneOfTagged = definitions:
    types.oneOf (lib.attrValues (lib.mapAttrs taggedSubmodule definitions));
in
{
  options = {
    services.kitsune = {
      enable = mkEnableOption ''
        Kitsune: an open-source social media server utilising the ActivityPub protocol.
      '';

      packages = {
        service = mkOption {
          type = types.package;
          default = pkgs.kitsune;
        };
      };

      dataDir = mkOption {
        type = types.path;
        default = "/var/lib/kitsune";
        readOnly = true;
      };

      config = mkOption {
        type = types.attrOf (types.nullOr [ types.bool types.int types.str ]);
        default = {
          database = {
            url = "postgres://kitsune:kitsune@localhost/kitsune";
            max-connections = 20;
          };
          job-queue = {
            redis-url = "redis+unix:///run/redis-kitsune-jobqueue/redis.sock";
            num-workers = 20;
          };
          messaging = {
            type = "in-process";
          };
        };
      };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ ];

    users.users.kitsune = {
      isSystemUser = true;
      group = "kitsune";
      extraGroups = [
        "redis-kitsune-jobqueue"
      ];
      home = cfg.dataDir;
    };

    users.groups.kitsune = { };

    services.redis = {
      package = pkgs.valkey;
      servers."kitsune-jobqueue".enable = true;
    };

    systemd.services.kitsune = {
      wantedBy = [ "multi-user.target" ];
      after = [
        "network.target"
        "postgresql.service"
        "redis-kitsune-jobqueue.service"
      ];

      wants = [ "network-online.target" ];

      serviceConfig = {
        User = "kitsune";
        Group = "kitsune";
        Restart = "always";
        # Necessary because /public routes are served cwd relative
        WorkingDirectory = "${cfg.packages.service}";
        ExecStartPre = "${pkgs.coreutils}/bin/mkdir -p ${cfg.dataDir}/uploads";
        ExecStart = "${cfg.packages.service}/bin/kitsune ${configFile}";
      };
    };
  };
}
