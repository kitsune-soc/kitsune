# kitsune

Kitsune is an open-souce social media server utilising the ActivityPub protocol. 
Utilising the capabilities of ActivityPub, you can interact with people on Mastodon, Misskey, Akkoma, etc. 
Due to its decentralised nature, you can full self-host Kitsune on your own hardware and still interact with everyone!

Kitsune itself is pretty lightweight and should run even on a Raspberry Pi (even though we haven't tested that yet).

## Installation

Besides the binary, you need the following services to run Kitsune:

- Redis (for caching)
- Meilisearch (for full-text search)
- (optional) a database server (either PostgreSQL or MySQL/MariaDB; you can use SQLite instead)
- NGINX (as a reverse-proxy)
- TLS certificate (we recommend a free one from "Let's Encrypt")

There are no good installation docs at the moment, but the installation itself is pretty straightforward. 
Rename `.env.example` to `.env`, change the values to fit your needs. The reverse proxy config is pretty vanilla, just forward the traffic. 
Check the Meilisearch docs on how to configure it correctly. The Redis installation is expected to be in a single-node configuration.

## Contributing

Contributions are very welcome. However, if you intend to change anything more than updating a dependency or fixing a small bug, please open an issue first. 
We would like to discuss any bigger changes before they are actually implemented.

### Note on required libraries

We use [Nix](https://nixos.org) for handling all our dependencies. 
When in doubt, install Nix and run `nix develop` to get yourself a shell with all required dependencies and services (you might need to enable some unstable features).

## License

Kitsune is licensed under the [MIT license](http://opensource.org/licenses/MIT).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, 
shall be licensed as above, without any additional terms or conditions.
