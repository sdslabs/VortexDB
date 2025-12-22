### Build

Clone the repository and run `cargo build --bin grpc_server` to build the binary of the gRPC server crate.

You can than then start the gRPC server by running:

```bash
cargo run --bin grpc_server
```

### Configuration

Use the [.sample.env](.sample.env) shown below as a reference to set your environment variables in a `.env` file.

```bash
GRPC_SERVER_ROOT_PASSWORD=123 # required
GRPC_SERVER_DIMENSION=3 # required

GRPC_SERVER_HOST=localhost # defaults to 127.0.0.1 aka localhost
GRPC_SERVER_PORT=8080 # defaults to 8080
GRPC_SERVER_STORAGE_TYPE=inmemory # (inmemory/rocksdb) defaults to 'inmemory'
GRPC_SERVER_INDEX_TYPE=flat # defaults to flat
GRPC_SERVER_DATA_PATH=data # defaults to a temporary directory
GRPC_SERVER_LOGGING=true # defaults to true


```


### Testing

The [vector-db.proto](proto/vector-db.proto) can be imported into any gRPC client.
