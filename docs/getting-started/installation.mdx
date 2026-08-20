---
title: "Installation"
description: "Get VortexDB up and running in minutes"
---

# Installation

VortexDB can be installed using Docker (recommended) or built from source. This guide covers both methods.

## Prerequisites

<CardGroup cols={2}>
  <Card title="Docker" icon="docker">
    Docker 20.10+ and Docker Compose v2
  </Card>
  <Card title="From Source" icon="rust">
    Rust 1.88+, protobuf-compiler, clang
  </Card>
</CardGroup>

## Docker Installation (Recommended)

The fastest way to get started is using Docker:

```bash
# Clone the repository
git clone https://github.com/sdslabs/VortexDB.git
cd VortexDB

# Copy the environment template
cp .env.example .env
```

### Configure Environment

Edit the `.env` file with your settings:

```bash
# Required settings
VORTEXDB_KEYS_FILE=./keys.json  # see keys.example.json for the format
DIMENSION=384  # Vector dimension (e.g., 384 for MiniLM embeddings)
DATA_PATH=/data

# Optional settings (with defaults)
HTTP_HOST=127.0.0.1
HTTP_PORT=3000
GRPC_HOST=127.0.0.1
GRPC_PORT=50051
STORAGE_TYPE=inmemory    # inmemory | rocksdb
INDEX_TYPE=flat          # flat | kdtree | hnsw
SIMILARITY=cosine        # cosine | euclidean | manhattan | hamming
LOGGING=true
DISABLE_HTTP=false
```

### Start VortexDB

```bash
docker compose up
```

<Tip>
Use `docker compose up --build` after making code changes to rebuild the image.
</Tip>

VortexDB is now running:
- **HTTP API**: http://localhost:3000
- **gRPC API**: localhost:50051

## Building from Source

### Install Dependencies

<Tabs>
  <Tab title="Ubuntu/Debian">
    ```bash
    sudo apt-get update && sudo apt-get install -y \
        protobuf-compiler \
        clang \
        libclang-dev \
        llvm-dev \
        build-essential
    ```
  </Tab>
  <Tab title="macOS">
    ```bash
    brew install protobuf llvm
    ```
  </Tab>
  <Tab title="Arch Linux">
    ```bash
    sudo pacman -S protobuf clang llvm
    ```
  </Tab>
</Tabs>

### Build and Run

```bash
# Clone the repository
git clone https://github.com/sdslabs/VortexDB.git
cd VortexDB

# Build in release mode
cargo build --release

# Run the server
./target/release/server
```

## Python SDK Installation

Install the Python client to interact with VortexDB:

```bash
pip install vortexdb
```

Or install from source:

```bash
cd client/python
pip install -e .
```

<Note>
The Python SDK requires Python 3.9+ and communicates with VortexDB via gRPC.
</Note>

## Verify Installation

Check that VortexDB is running correctly:

<CodeGroup>
```bash Health Check (HTTP)
curl http://localhost:3000/health
# Response: OK
```

```python Health Check (Python)
from vortexdb import VortexDB

db = VortexDB(
    grpc_url="localhost:50051",
    api_key="your-secure-password"
)
print("Connected successfully!")
db.close()
```
</CodeGroup>

## Next Steps

<Card title="Quickstart" icon="rocket" href="/getting-started/quickstart">
  Insert your first vector in 60 seconds
</Card>
