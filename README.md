## Quick Start

This project uses a **Makefile** to simplify common tasks like building and running the node.

### Setup

Install and configure the Rust toolchain for **WebAssembly compilation**:

```bash
make init
```

### Build & Run

* **Build only:** Compile the node without launching it:

```bash
make build
```

* **Run in development mode:** Launch a temporary node (state will be discarded when stopped):

```bash
make run
```

### Explore Commands

After building, check all available parameters and subcommands:

```bash
./target/release/paralink-node -h
```

---

## Running the Node

### Single-Node Development Chain

Start a single-node chain with **persistent state**:

```bash
./target/release/paralink-node --dev
```

Reset the chain state if needed:

```bash
./target/release/paralink-node purge-chain --dev
```

Enable **detailed logging** for debugging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/paralink-node -lruntime=debug --dev
```

---

### Running with Docker

Use the included script to run the node inside a Docker container:

```bash
./scripts/docker_run.sh
```

By default, this will **compile the code** and launch a local development network. You can also customize commands:

```bash
# Run the node without recompiling
./scripts/docker_run.sh ./target/release/paralink-node --dev --ws-external

# Purge local development chain
./scripts/docker_run.sh ./target/release/paralink-node purge-chain --dev

# Check compilation without building
./scripts/docker_run.sh cargo check
```


