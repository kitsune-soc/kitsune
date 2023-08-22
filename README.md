<div align="center">

# kitsune

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/kitsune-soc/kitsune/rust.yml?style=for-the-badge)
[![dependency status](https://deps.rs/repo/github/kitsune-soc/kitsune/status.svg?style=for-the-badge)](https://deps.rs/repo/github/kitsune-soc/kitsune)

</div>

Kitsune is an open-souce social media server utilising the ActivityPub protocol.
Utilising the capabilities of ActivityPub, you can interact with people on Mastodon, Misskey, Akkoma, etc.
Due to its decentralised nature, you can self-host Kitsune on your own hardware and still interact with everyone!

Kitsune itself is pretty lightweight and should run even on a Raspberry Pi (even though we haven't tested that yet).

**[Documentation](https://docs.joinkitsune.org)**

### Chat

Both chat options are bridged. Feel free to join whichever you're more comfortable with:

[![Matrix](https://img.shields.io/matrix/kitsune-space:matrix.org?label=Matrix%20chat&style=for-the-badge)](https://matrix.to/#/#kitsune-space:matrix.org)
[![Discord](https://img.shields.io/discord/1118538521423138856?label=Discord%20chat&style=for-the-badge)](https://discord.gg/YGAtX7nfrG)

## ⚠ Disclaimer

This software is far from production-ready. Breaking changes might happen.
So, as long as this disclaimer is here, make sure to double check all the changes before you update your installation.

## Project structure

- `contrib/`: Files for configuring Kitsune (Systemd, Caddy, etc.)
- `crates/`: Sub-crates Kitsune consists of
- `kitsune/`: Main Kitsune server binary
- `kitsune-cli/`: Kitsune CLI binary
- `kitsune-fe/`: Kitsune frontend project
- `kitsune-search-server/`: Search server for Kitsune
- `lib/`: Libraries made for Kitsune but with no dependencies on Kitsune-specific code. Easily usable by other projects
- `proto/`: Any Protobuf definitions for the Kitsune project
- `public/`: Public web assets

## State of federation

We successfully federated with Mastodon on the following functionality:

- Webfinger
- Actors
- Posts
  - Reposts
    - Incoming
  - Create
    - Incoming and outgoing
  - Delete
    - Incoming and outgoing
  - Replies
    - Incoming and outgoing
  - Content Warnings
    - Outgoing
  - Media attachments
    - Outgoing
- Likes
  - Added
    - Incoming and outgoing
  - Removed
    - Incoming and outgoing
- Follows
  - Added
    - Incoming and outgoing
  - Removed
    - Incoming and outgoing

(last updated: 26.05.2023)

## Contributing

Contributions are very welcome. However, if you intend to change anything more than updating a dependency or fixing a small bug, please open an issue first.
We would like to discuss any bigger changes before they are actually implemented.

### Security

If you found a suspected security vulnerability, please refer to our [security policy](./SECURITY.md) for more details.

### Note on required libraries

We use [Nix](https://nixos.org) for handling our development dependencies.
When in doubt, install Nix and run `nix develop` to get yourself a shell with all required dependencies and services (you might need to enable some unstable features).

## License

Kitsune is licensed under the [MIT license](http://opensource.org/licenses/MIT).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
shall be licensed as above, without any additional terms or conditions.
