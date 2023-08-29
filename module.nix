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
        type = types.path;
        default = "/var/lib/kitsune";
        readOnly = true;
      };

      config = mkOption {
        type = types.submodule {
          freeformType = format.type;
          options = {
            cache = mkOption {
              type = oneOfTagged {
                in-memory = { };
                redis = {
                  url = mkOption {
                    type = types.nonEmptyStr;
                    default = "redis+unix:///run/redis-kitsune-cache/redis.sock";
                  };
                };
              };
              default = {
                type = "redis";
              };
            };
            database = mkOption {
              type = types.submodule {
                freeformType = format.type;
                options = {
                  url = mkOption {
                    type = types.nonEmptyStr;
                    default = "postgres://kitsune:kitsune@localhost/kitsune";
                  };
                  max-connections = mkOption {
                    type = types.ints.positive;
                    default = 20;
                  };
                };
              };
              default = { };
            };
            instance = mkOption {
              type = types.submodule {
                freeformType = format.type;
                options = {
                  name = mkOption {
                    type = types.str;
                  };
                  description = mkOption {
                    type = types.str;
                  };
                  character-limit = mkOption {
                    type = types.ints.positive;
                    default = 5000;
                  };
                  registrations-open = mkOption {
                    type = types.bool;
                  };
                  federation-filter = mkOption {
                    type = oneOfTagged {
                      deny = {
                        domains = mkOption {
                          type = types.listOf types.str;
                          default = [ ];
                        };
                      };
                      allow = {
                        domains = mkOption {
                          type = types.listOf types.str;
                          default = [ ];
                        };
                      };
                    };
                    default = {
                      type = "deny";
                    };
                  };
                };
              };
            };
            job-queue = mkOption {
              type = types.submodule {
                freeformType = format.type;
                options = {
                  redis-url = mkOption {
                    type = types.nonEmptyStr;
                    default = "redis+unix:///run/redis-kitsune-jobqueue/redis.sock";
                  };
                  num-workers = mkOption {
                    type = types.ints.positive;
                    default = 20;
                  };
                };
              };
              default = { };
            };
            messaging = mkOption {
              type = oneOfTagged {
                in-process = { };
                redis = {
                  url = mkOption {
                    type = types.nonEmptyStr;
                    default = "redis+unix:///run/redis-kitsune-messaging/redis.sock";
                  };
                };
              };
              default = {
                type = "redis";
              };
            };
            server = mkOption {
              type = types.submodule {
                freeformType = format.type;
                options = {
                  frontend-dir = mkOption {
                    type = types.path;
                    default = "${cfg.packages.service}/kitsune-fe";
                  };
                  max-upload-size = mkOption {
                    type = types.ints.positive;
                    default = 5242880;
                  };
                  media-proxy-enabled = mkOption {
                    type = types.bool;
                    default = false;
                  };
                  port = mkOption {
                    type = types.port;
                    default = 5000;
                  };
                  prometheus-port = mkOption {
                    type = types.port;
                    default = 9000;
                  };
                  request-timeout-secs = mkOption {
                    type = types.ints.positive;
                    default = 60;
                  };
                };
              };
              default = { };
            };
            search = mkOption {
              type = oneOfTagged {
                kitsune = {
                  index-server = mkOption {
                    type = types.nonEmptyStr;
                  };
                  search-servers = mkOption {
                    type = types.listOf types.nonEmptyStr;
                  };
                };
                meilisearch = {
                  instance-url = mkOption {
                    type = types.nonEmptyStr;
                  };
                };
                sql = { };
                none = { };
              };
              default = { type = "sql"; };
            };
            storage = mkOption {
              type = oneOfTagged {
                fs = {
                  upload-dir = mkOption {
                    type = types.path;
                    default = "${cfg.dataDir}/uploads";
                  };
                };
                s3 = {
                  todo = mkOption {
                    type = types.enum [ ];
                  };
                };
              };
              default = {
                type = "fs";
              };
            };
            url = mkOption {
              type = types.submodule {
                freeformType = format.type;
                options = {
                  scheme = mkOption {
                    type = types.nonEmptyStr;
                  };
                  domain = mkOption {
                    type = types.nonEmptyStr;
                  };
                };
              };
              default = { };
            };
          };
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
        "redis-kitsune-messaging"
      ];
      home = cfg.dataDir;
    };

    users.groups.kitsune = { };

    services.redis = {
      servers."kitsune-cache".enable = true;
      servers."kitsune-jobqueue".enable = true;
      servers."kitsune-messaging".enable = true;
    };

    systemd.services.kitsune = {
      wantedBy = [ "multi-user.target" ];
      after = [
        "network.target"
        "postgresql.service"
        "redis-kitsune-cache.service"
        "redis-kitsune-jobqueue.service"
        "redis-kitsune-messaging.service"
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
