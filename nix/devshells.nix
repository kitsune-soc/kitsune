{ devenv, pkgs, inputs }:
rec {
  default = backend;

  backend = devenv.lib.mkShell {
    inherit pkgs inputs;

    modules = [
      (
        { pkgs, ... }:
        {
          packages =
            with pkgs;
            [
              cargo-insta
              diesel-cli
              rust-bin.stable.latest.default
            ];

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
            redis = {
              package = pkgs.valkey;
              enable = true;
            };
          };
        }
      )
    ];
  };

  frontend = pkgs.mkShell {
    buildInputs = with pkgs; [
      nodejs
      nodePackages.svelte-language-server
      nodePackages.typescript-language-server
      pnpm
    ];
  };
}
