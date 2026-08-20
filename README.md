<p align="center">
  <img src="assets/logo.svg" alt="VortexDB Logo" height="100">
</p>

<p align="center">
  <a href="https://github.com/sdslabs/VortexDB/actions"><img src="https://img.shields.io/github/actions/workflow/status/sdslabs/VortexDB/rust.yml?branch=refactor&style=flat-square&label=build" alt="Build Status"></a>
  <a href="#"><img src="https://img.shields.io/badge/docs-coming%20soon-yellow?style=flat-square" alt="Docs"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square" alt="License"></a>
  <a href="https://discord.gg/dXkVEgTPu9"><img src="https://img.shields.io/discord/758961084337618944?style=flat-square&logo=discord&label=discord" alt="Discord"></a>
</p>

**VortexDB** - A vector database built from scratch in Rust 🦀. VortexDB is designed for efficient similarity search and can be used as the backbone for AI-powered applications.

---

## Architecture

VortexDB is organized as a Rust workspace with modular crates for flexibility and maintainability.

### Indexers

The indexing layer provides efficient vector similarity search with pluggable index implementations. VortexDB supports **Flat** indexing for brute-force exact search (ideal for smaller datasets), **KD-Tree** for space-partitioning in low-dimensional vectors, and **HNSW** (Hierarchical Navigable Small World) graphs for fast approximate nearest neighbor search on large-scale datasets.

Supported distance metrics include Euclidean, Manhattan, Hamming, and Cosine similarity.

### Storage Engine

The storage layer abstracts persistence with a `StorageEngine` trait, allowing different backends. **RocksDB** offers persistent, production-ready storage for real-world deployments.



## Clients

### HTTP Server

RESTful API server built with [Axum](https://github.com/tokio-rs/axum):

```
GET  /           - Root endpoint
GET  /health     - Health check
POST /points     - Insert a point
GET  /points/:id - Get a point by ID
DELETE /points/:id - Delete a point
POST /points/search - Search for similar vectors
```

### gRPC Server

High-performance gRPC server with Protocol Buffers. Features include:
- Configurable logging
- Full CRUD operations for vector points

### TUI Client

Interactive terminal user interface built with [Ratatui](https://github.com/ratatui/ratatui):
- Dashboard view for database overview
- Database management
- Vector operations (insert, search, delete)
- Modal dialogs for user input

### Unified Server Startup

VortexDB offers a Dockerfile and a docker-compose for starting up the gRPC and HTTP servers  
Steps to get the unified server running:  
```
cp .env.example .env
```

The following env vars are important, and are required to be set by the user:  
`GRPC_ROOT_PASSWORD`  
`DIMENSION`  
`DATA_PATH`  

**NOTE**: `DATA_PATH` is the directory within the container where persistent data is stored


Setting of the following env vars is optional, as they fallback to safe defaults, but recommended:  
| .env Var | Function | Safe Default |
| :--- | :--- | :--- |
| `HTTP_HOST` | Host IP for the HTTP Server | `0.0.0.0` |
| `HTTP_PORT` | Port for the HTTP Server | `3000` |
| `GRPC_HOST` | Host IP for the gRPC Server | `0.0.0.0` |
| `GRPC_PORT` | Port for the gRPC Server | `50051` |
| `STORAGE_TYPE` | Store `inmemory` \| `rocksdb` | `inmemory` |
| `INDEX_TYPE` | Type of Indexer: `flat` \| `kdtree` \| `hnsw` | `flat` |
| `LOGGING` | Enable logs | `true` |
| `DISABLE_HTTP` | Bool flag to run the HTTP Server | `false` |


**NOTE**: CLI flags (e.g. `docker compose run --env` ) take precedence over docker-compose environment values, which in turn override `.env` file variables.

Now you're all set up.  

To run the unified server:  
```
docker compose up
```
To build the image again and run the unified server (recommended after code changes):
```
docker compose up --build
```
To run only the gRPC server (HTTP server disabled):
```
DISABLE_HTTP=true docker compose up
```



## Roadmap

- [ ] InMemory implementation of StorageEngine
- [ ] Snapshots for indexers
- [ ] Benchmarking suite
- [ ] Python SDK
- [ ] JavaScript/TypeScript SDK
- [ ] Modular vectorization service


## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.