{ config, lib, pkgs, ... }:
let
  inherit (lib) types mkEnableOption mkOption;
  inherit (builtins) toJSON;
  cfg = config.services.kitsune;
  configFile = pkgs.writeText "config.dhall" ''
    let types = ${cfg.packages.service}/config/types.dhall
    in    { cache =
                types.Cache.Redis { redis_url = "redis+unix:///run/redis-kitsune-cache/redis.sock" }
              : types.Cache
          , database =
                { url = "postgres://kitsune:kitsune@localhost/kitsune", max_connections = 20 }
              : types.Database
          , email = None types.Email
          , embed = None types.Embed
          , instance =
                { name = ${toJSON cfg.name}
                , description = ${toJSON cfg.description}
                , character_limit = 5000
                , federation_filter =
                      types.FederationFilter.Deny { domains = [] : List Text }
                    : types.FederationFilter
                , registrations_open = ${if cfg.registrationsOpen then "True" else "False"}
                }
              : types.Instance
          , job_queue = { redis_url = "redis+unix:///run/redis-kitsune-jobqueue/redis.sock" } : types.JobQueue
          , messaging = types.Messaging.InProcess
          , server =
                { frontend_dir = "${cfg.packages.service}/kitsune-fe"
                , job_workers = 20
                , max_upload_size = 5 * 1024 * 1024
                , media_proxy_enabled = False
                , oidc = None types.Oidc
                , port = 5000
                , prometheus_port = 9000
                , request_timeout_sec = 60
                }
              : types.Server
          , search = types.Search.Sql
          , storage = types.Storage.Fs { upload_dir = ${toJSON "${cfg.dataDir}/uploads"} } : types.Storage
          , url = { scheme = ${toJSON cfg.url.scheme}, domain = ${toJSON cfg.url.domain} } : types.Url
          }
        : types.Config
  '';
in {
  options = {
    services.kitsune = {
      enable = mkEnableOption ''
        Kitsune: an open-souce social media server utilising the ActivityPub protocol.
      '';

      packages = {
        service = mkOption {
          type = types.package;
          default = pkgs.kitsune;
        };
        cli = mkOption {
          type = types.package;
          default = pkgs.kitsune-cli;
        };
      };

      dataDir = mkOption {
        type = types.str;
        default = "/var/lib/kitsune";
        readOnly = true;
      };

      name = mkOption {
        type = types.str;
      };

      description = mkOption {
        type = types.str;
      };

      registrationsOpen = mkOption {
        type = types.bool;
      };

      url = {
        scheme = mkOption {
          type = types.str;
        };

        domain = mkOption {
          type = types.str;
        };
      };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.packages.cli ];

    users.users.kitsune = {
      isSystemUser = true;
      group = "kitsune";
      extraGroups = [
        "redis-kitsune-cache"
        "redis-kitsune-jobqueue"
      ];
      home = cfg.dataDir;
    };

    users.groups.kitsune = { };

    services.redis = {
      servers."kitsune-cache".enable = true;
      servers."kitsune-jobqueue".enable = true;
    };

    systemd.services.kitsune = {
      wantedBy = [ "multi-user.target" ];
      after = [
        "network.target"
        "postgresql.service"
        "redis-kitsune-cache.service"
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
