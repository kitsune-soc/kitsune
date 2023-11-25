# Installation

At the moment, the only way to install Kitsune is to compile it from source.  
Don't worry, that sounds way more scary than it actually is. In this guide we will step you through it.

## Dependencies

In order to build Kitsune, need a few dependencies. These are:

1. The Rust toolchain ([recommended installation](https://rustup.rs/))
2. PostgreSQL as a dedicated DBMS
3. Redis for the job queue
4. [NodeJS](https://nodejs.org/en) v16+
5. [Yarn](https://classic.yarnpkg.com/en/docs/install)
6. Reverse Proxy (recommended: [Caddy](https://caddyserver.com/docs/install))

Yes, that's really it. We don't need more. Kitsune is designed to use as few native dependencies as possible to make building from source easy!

### Note on C/C++ compiler requirement

Rust needs a C/C++ compiler to invoke the linker and potentially build some native libraries to statically link to, so make sure you have one installed.

To install it under Debian and Ubuntu, run the following command:

```bash
sudo apt install build-essential
```

The name of the package(s) containing these tools might differ between distributions.

## Building

First, grab yourself a copy of the source code. You can either download the [main branch ZIP archive](https://github.com/kitsune-soc/kitsune/archive/refs/heads/main.zip) or clone it via Git. Both ways will work just fine.

> The recommended way to download the source is via Git since it makes updating and doing a rollback way easier.

To download a Git repository, just run the following command (this assumes you have [Git](https://git-scm.com/) installed):

```bash
git clone https://github.com/kitsune-soc/kitsune.git
```

> If you downloaded the ZIP, just unzip the archive

Next, move into the newly created directory and then into the `kitsune-fe` directory and build the frontend. 
To do this run:

```bash
yarn install && yarn build
```

Now move out of the directory and back into the main one. Then build the binaries in release mode.  
To do this run the following command:

```bash
cargo build --release
```

After the command finished there should be the following three binaries inside the `target/release` directory:

- `kitsune`: The main Kitsune application. This is the heart of the whole application.
- `kitsune-cli`: The Kitsune CLI utility. Used to give users roles and clear the job scheduler table.
- `kitsune-job-runner`: The dedicated Kitsune job runner

That's it for the building part but before we can actually run our instance, we need to configure it first.
