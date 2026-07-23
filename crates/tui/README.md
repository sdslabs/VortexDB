# VortexDB TUI

A terminal user interface for managing VortexDB databases locally.

## Important Note

> **This is currently a local admin/development tool**, not a network client. The TUI operates directly on database files using embedded storage — it does not connect to a running VortexDB server. This will change once the server gets the multiple-database support.
>
> For remote server access, use the [Python client](../../client/python/).

## Usage

```bash
cargo run -p tui
```

Databases are stored in `./databases/` by default.

### Environment Variables

For embedding features, set these in your `.env` file or environment:

```bash
TEXT_EMBEDDING_URL=http://localhost:8000/embed/text
IMAGE_EMBEDDING_URL=http://localhost:8000/embed/image
```

These point to an external embedding service that generates vectors from text/images.

## Roadmap

This TUI will evolve into a full client that connects to VortexDB servers over gRPC. Planned changes:

- Add remote connection mode via gRPC (like the Python client)
- Move to `client/tui/` once server multi-database support lands
