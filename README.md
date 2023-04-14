<div align="center">

# kitsune

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/kitsune-soc/kitsune/rust.yml?style=for-the-badge)
[![dependency status](https://deps.rs/repo/github/kitsune-soc/kitsune/status.svg?style=for-the-badge)](https://deps.rs/repo/github/kitsune-soc/kitsune)
![Matrix](https://img.shields.io/matrix/kitsune-space:matrix.org?label=Matrix%20chat&style=for-the-badge)

</div>

Kitsune is an open-souce social media server utilising the ActivityPub protocol. 
Utilising the capabilities of ActivityPub, you can interact with people on Mastodon, Misskey, Akkoma, etc. 
Due to its decentralised nature, you can self-host Kitsune on your own hardware and still interact with everyone!

Kitsune itself is pretty lightweight and should run even on a Raspberry Pi (even though we haven't tested that yet).

**[Documentation](https://docs.joinkitsune.org)**

## ⚠ Disclaimer

This software is far from production-ready. Breaking changes might happen. 
So, as long as this disclaimer is here, make sure to double check all the changes before you update your installation.

## State of federation

We successfully federated with Mastodon on the following functionality:

- Webfinger
- Actors
- Posts
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
    - Removed
- Follows
    - Added
        - Incoming
    - Removed
        - Incoming

(last updated: 09.04.2023)

## Contributing

Contributions are very welcome. However, if you intend to change anything more than updating a dependency or fixing a small bug, please open an issue first. 
We would like to discuss any bigger changes before they are actually implemented.

### Note on required libraries

We use [Nix](https://nixos.org) for handling our development dependencies. 
When in doubt, install Nix and run `nix develop` to get yourself a shell with all required dependencies and services (you might need to enable some unstable features).

## License

Kitsune is licensed under the [MIT license](http://opensource.org/licenses/MIT).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, 
shall be licensed as above, without any additional terms or conditions.
