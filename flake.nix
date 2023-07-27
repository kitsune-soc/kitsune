{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, flake-utils, nixpkgs, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit overlays system;
        };
        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.rust-bin.stable.latest.minimal;
          rustc = pkgs.rust-bin.stable.latest.minimal;
        };
        baseDependencies = with pkgs; [
          openssl
          pkg-config
          protobuf
          sqlite
          zlib
        ];
        cargoConfig = builtins.fromTOML (builtins.readFile ./.cargo/config.toml);  # TODO: Set the target CPU conditionally
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

          nativeBuildInputs = baseDependencies;

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig"; # Not sure why this is broken but it is
          RUSTFLAGS = builtins.concatStringsSep " " cargoConfig.build.rustflags; # Oh god help.

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
          ];
        };
      in
      {
        packages = rec {
          default = main;
          cli = rustPlatform.buildRustPackage (basePackage // {
            pname = "kitsune-cli";
            cargoBuildFlags = "-p kitsune-cli";
          });
          main = rustPlatform.buildRustPackage (basePackage // {
            pname = "kitsune";
            buildFeatures = [ "meilisearch" ];
            cargoBuildFlags = "-p kitsune";
          });
          search = rustPlatform.buildRustPackage (basePackage // {
            pname = "kitsune-search";
            cargoBuildFlags = "-p kitsune-search";
          });
          frontend = pkgs.mkYarnPackage {
            inherit version;

            src = "${src}/kitsune-fe";

            buildPhase = ''
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
          backend = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo-insta
              diesel-cli
              redis
              rust-bin.stable.latest.default
            ]
            ++
            baseDependencies;
          };
          backend-full = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo-insta
              diesel-cli
              redis
              postgresql
              rust-bin.stable.latest.default
            ]
            ++
            baseDependencies;
            shellHook = ''
              source ./.devshell_prompt.sh

              export PG_HOST=127.0.0.3
              export PG_PORT=45999
              export DATABASE_URL=postgres://$USER@$PG_HOST:$PG_PORT/$USER

              #something more sophisticated to kill OUR postgres and not a system one?
              #like using the lockfile it makes or a job we start?
              export PG_DIR=data
              POSTGRES_INVOCATION="postgres -D $PG_DIR -h $PG_HOST -p $PG_PORT -k ."

              pkill -f "$POSTGRES_INVOCATION" 
              pidwait -f "$POSTGRES_INVOCATION" 
              
              mkdir -p $PG_DIR
              rm -rf $PG_DIR/*
              initdb -D $PG_DIR --no-locale --encoding=UTF8 >/dev/null

              # setsid is here so that ctrl+c in the shell does not kill the server. 
              # For some reason only setsid works, no nohup or stdin redir
              setsid $POSTGRES_INVOCATION 2>/dev/null >/dev/null &
              sleep 3
              createdb -h $PG_HOST -p $PG_PORT $USER
              
              export REDIS_PORT=6379
              export REDIS_URL="redis://127.0.0.1:$REDIS_PORT"

              pkill -f "redis-server 127.0.0.1:$REDIS_PORT"
              pidwait -f "redis-server 127.0.0.1:$REDIS_PORT"
              setsid  redis-server --bind 127.0.0.1 --port $REDIS_PORT >/dev/null &
              
            '';
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
