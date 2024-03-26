{
  inputs = {
    devenv = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:cachix/devenv";
    };

    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    rust-overlay = {
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
      url = "github:oxalica/rust-overlay";
    };
  };
  outputs = { self, devenv, flake-utils, nixpkgs, rust-overlay, ... } @ inputs:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit overlays system;
          };
          stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = pkgs.rust-bin.stable.latest.minimal;
            rustc = pkgs.rust-bin.stable.latest.minimal;
            inherit stdenv;
          };
          baseDependencies = with pkgs; [
            openssl
            sqlite
            zlib
          ];
          nativeDependencies = with pkgs; [
            protobuf
            pkg-config
            rustPlatform.bindgenHook
          ];

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          src = pkgs.lib.cleanSourceWith {
            src = pkgs.lib.cleanSource ./.;
            filter = name: type:
              let baseName = baseNameOf (toString name);
              in !(baseName == "flake.lock" || pkgs.lib.hasSuffix ".nix" baseName);
          };
          version = cargoToml.workspace.package.version;

          basePackage = {
            inherit version src;

            meta = {
              description = "ActivityPub-federated microblogging";
              homepage = "https://joinkitsune.org";
            };

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            OPENSSL_NO_VENDOR = 1;

            buildInputs = baseDependencies;

            nativeBuildInputs = nativeDependencies;

            checkFlags = [
              # Depend on creating an HTTP client and that reads from the systems truststore
              # Because nix is fully isolated, these types of tests fail
              #
              # Some (most?) of these also depend on the network? Not good??
              "--skip=activitypub::fetcher::test::federation_allow"
              "--skip=activitypub::fetcher::test::federation_deny"
              "--skip=activitypub::fetcher::test::fetch_actor"
              "--skip=activitypub::fetcher::test::fetch_note"
              "--skip=resolve::post::test::parse_mentions"
              "--skip=webfinger::test::fetch_qarnax_ap_id"
              "--skip=basic_request"
              "--skip=json_request"
              "--skip=http::handler::well_known::webfinger::tests::basic"
              "--skip=http::handler::well_known::webfinger::tests::custom_domain"
              "--skip=test::default_resolver_works"
              "--skip=fetcher::basic::fetch_actor"
              "--skip=fetcher::basic::fetch_emoji"
              "--skip=fetcher::basic::fetch_note"
              "--skip=fetcher::filter::federation_allow"
              "--skip=fetcher::filter::federation_deny"
              "--skip=fetcher::infinite::fetch_infinitely_long_reply_chain"
              "--skip=fetcher::origin::check_ap_content_type"
              "--skip=fetcher::origin::check_ap_id_authority"
              "--skip=fetcher::webfinger::fetch_actor_with_custom_acct"
              "--skip=fetcher::webfinger::ignore_fake_webfinger_acct"
              "--skip=accounts_username"
              "--skip=users_username"
              "--skip=test::abort_request_works"
              "--skip=test::full_test"
            ];
          };
        in
        {
          formatter = pkgs.nixpkgs-fmt;
          packages = rec {
            # Hack to make latest devenv work
            devenv-up = self.devShells.${system}.default.config.procfileScript;

            default = main;

            cli = rustPlatform.buildRustPackage (basePackage // {
              pname = "kitsune-cli";
              cargoBuildFlags = "-p kitsune-cli";
            });

            main = rustPlatform.buildRustPackage (basePackage // {
              pname = "kitsune";
              buildFeatures = [ "meilisearch" "oidc" ];
              cargoBuildFlags = "-p kitsune";
            });

            frontend = pkgs.mkYarnPackage {
              inherit version;
              packageJSON = "${src}/kitsune-fe/package.json";
              yarnLock = "${src}/kitsune-fe/yarn.lock";
              src = "${src}/kitsune-fe";

              buildPhase = ''
                export HOME=$(mktemp -d)
                yarn --offline build
              '';

              installPhase = ''
                mkdir -p $out
                cp -R deps/kitsune-fe/dist $out
              '';

              distPhase = "true";
            };
          };

          devShells = rec {
            default = backend;

            backend = devenv.lib.mkShell {
              inherit pkgs inputs;

              modules = [
                ({ pkgs, ... }: {
                  packages = with pkgs; [
                    cargo-insta
                    diesel-cli
                    rust-bin.stable.latest.default
                  ]
                  ++
                  baseDependencies ++ nativeDependencies;

                  enterShell = ''
                    export PG_HOST=127.0.0.1
                    export PG_PORT=5432
                    [ -z "$DATABASE_URL" ] && export DATABASE_URL=postgres://$USER@$PG_HOST:$PG_PORT/$USER

                    export REDIS_PORT=6379
                    [ -z "$REDIS_URL" ] && export REDIS_URL="redis://127.0.0.1:$REDIS_PORT"
                  '';

                  services = {
                    postgres = {
                      enable = true;
                      listen_addresses = "127.0.0.1";
                    };
                    redis.enable = true;
                  };
                })
              ];
            };

            frontend = pkgs.mkShell {
              buildInputs = with pkgs; [
                nodejs
                yarn
              ];
            };
          };
        }
      ) // {
      overlays = rec {
        default = kitsune;
        kitsune = (import ./overlay.nix self);
      };

      nixosModules = rec {
        default = kitsune;
        kitsune = (import ./module.nix);
      };
    };
}
