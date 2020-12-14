![](https://paralink.network/images/logo-sm-home.png)

## Getting Started

This repository contains Substrate based runtime for Paralink Network.

### Makefile

This project uses a [Makefile](Makefile) to document helpful commands and make it easier to execute them.

1. `make init` - Run the [init script](scripts/init.sh) to configure the Rust toolchain for
   [WebAssembly compilation](https://substrate.dev/docs/en/knowledgebase/getting-started/#webassembly-compilation).
1. `make run` - Build and launch this project in development mode.

### Build

The `make run` command will perform an initial build. Use the following command to build the node without launching it:

```sh
make build
```

### Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and subcommands:

```sh
./target/release/paralink-node -h
```

## Run

The `make run` command will launch a temporary node and its state will be discarded after you terminate the process. After the project has been built, there are other ways to launch the node.

### Single-Node Development Chain

This command will start the single-node development chain with persistent state:

```bash
./target/release/paralink-node --dev
```

Purge the development chain's state:

```bash
./target/release/paralink-node purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/paralink-node -lruntime=debug --dev
```

### Run in Docker

Then run the following command to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can also replace the default command (`cargo build --release && ./target/release/paralink-node --dev --ws-external`) by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling
./scripts/docker_run.sh ./target/release/paralink-node --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/paralink-node purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```
