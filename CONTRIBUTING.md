# Contributing

Contributions are very welcome. However, if you intend to change anything more than updating a dependency or fixing a small bug, please open an issue first.
We would like to discuss any bigger changes before they are actually implemented.

## Tech stack

Our tech stack mainly consists of the following:

- Rust
- PostgreSQL
- SvelteKit
- TypeScript
- TailwindCSS

## Finding issues to fix

If you are looking for issues to fix, you can look over the [issue tracker](https://github.com/kitsune-soc/kitsune/issues) and comment under the issue that interests you!  
We will get back to you, assign you the issue if you're up for it, and answer questions about the codebase in the issue or on Matrix!

## Project structure

- `contrib/`: Files for configuring Kitsune (Systemd, Caddy, etc.)
- `crates/`: Sub-crates Kitsune consists of
- `docs/`: Documentation in form of an [mdBook](https://rust-lang.github.io/mdBook/)
- `kitsune/`: Main Kitsune server binary
- `kitsune-cli/`: Kitsune CLI binary
- `kitsune-fe/`: Kitsune frontend project
- `kitsune-job-runner/`: Kitsune dedicated job runner
- `lib/`: Libraries made for Kitsune but with no dependencies on Kitsune-specific code. Easily usable by other projects
- `public/`: Public web assets
- `web/`: Resources specific to the [website](https://joinkitsune.org)
- `xtask/`: Task-runner polyfill

## Note on required libraries

We use [Nix](https://nixos.org) for handling our development dependencies.
When in doubt, install Nix and run `nix develop` to get yourself a shell with all required dependencies and services
(you might need to enable some unstable features of Nix since Flakes aren't stable yet!).
