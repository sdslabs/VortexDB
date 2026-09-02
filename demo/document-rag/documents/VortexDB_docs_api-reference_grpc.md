---
title: "gRPC API"
description: "High-performance Protocol Buffer API reference"
---

# gRPC API Reference

VortexDB's gRPC API provides high-performance vector operations using Protocol Buffers over HTTP/2.

## Connection

| Parameter | Default |
|-----------|---------|
| Host | `localhost` |
| Port | `50051` |
| Protocol | HTTP/2 (plaintext) |

```bash
# Test connection with grpcurl
grpcurl -plaintext localhost:50051 list
```

## Authentication

All gRPC calls require the `authorization` header with your API key:

```bash
-H "authorization: your-api-key"
```

Valid keys come from the JSON file pointed to by the `VORTEXDB_KEYS_FILE` environment variable, shared with the HTTP server. `readonly` keys can call `GetPoint`, `SearchPoints`, and `SearchPointsBatch`; `readwrite` keys can additionally call `InsertVector`, `InsertVectorsBatch`, and `DeletePoint`. A `readonly` key calling a write RPC gets a `PERMISSION_DENIED` status.

---

## Service Definition

```protobuf
syntax = "proto3";
package vectordb;

service VectorDB {
  rpc InsertVector(InsertVectorRequest) returns (PointID);
  rpc InsertVectorsBatch(InsertVectorsBatchRequest) returns (InsertVectorsBatchResponse);
  rpc DeletePoint(PointID) returns (google.protobuf.Empty);
  rpc GetPoint(PointID) returns (Point);
  rpc SearchPoints(SearchRequest) returns (SearchResponse);
  rpc SearchPointsBatch(SearchPointsBatchRequest) returns (SearchPointsBatchResponse);
}
```

---

## Methods

### InsertVector

Insert a vector with its associated payload.

<ParamField body="vector" type="DenseVector" required>
  The vector to insert. Must match the configured `DIMENSION`.
</ParamField>

<ParamField body="payload" type="Payload" required>
  Metadata associated with the vector.
</ParamField>

**Request:**

```protobuf
message InsertVectorRequest {
  DenseVector vector = 1;
  Payload payload = 2;
}
```

**Response:**

```protobuf
message PointID {
  UUID id = 1;
}
```

**Example:**

```bash
grpcurl -plaintext \
  -H "authorization: secret" \
  -d '{
    "vector": {"values": [0.1, 0.2, 0.3, 0.4]},
    "payload": {"content_type": 1, "content": "Hello world"}
  }' \
  localhost:50051 vectordb.VectorDB/InsertVector
```

**Response:**

```json
{
  "id": {
    "value": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

---

### GetPoint

Retrieve a point by its ID.

<ParamField body="id" type="UUID" required>
  The unique identifier of the point.
</ParamField>

**Request:**

```protobuf
message PointID {
  UUID id = 1;
}
```

**Response:**

```protobuf
message Point {
  PointID id = 1;
  Payload payload = 2;
  DenseVector vector = 3;
}
```

**Example:**

```bash
grpcurl -plaintext \
  -H "authorization: secret" \
  -d '{"id": {"value": "550e8400-e29b-41d4-a716-446655440000"}}' \
  localhost:50051 vectordb.VectorDB/GetPoint
```

**Response:**

```json
{
  "id": {
    "id": {
      "value": "550e8400-e29b-41d4-a716-446655440000"
    }
  },
  "payload": {
    "contentType": "Text",
    "content": "Hello world"
  },
  "vector": {
    "values": [0.1, 0.2, 0.3, 0.4]
  }
}
```

---

### DeletePoint

Delete a point by its ID.

<ParamField body="id" type="UUID" required>
  The unique identifier of the point to delete.
</ParamField>

**Request:**

```protobuf
message PointID {
  UUID id = 1;
}
```

**Response:**

```protobuf
google.protobuf.Empty
```

**Example:**

```bash
grpcurl -plaintext \
  -H "authorization: secret" \
  -d '{"id": {"value": "550e8400-e29b-41d4-a716-446655440000"}}' \
  localhost:50051 vectordb.VectorDB/DeletePoint
```

**Response:**

```json
{}
```

---

### SearchPoints

Search for the k nearest neighbors to a query vector.

<ParamField body="query_vector" type="DenseVector" required>
  The vector to search with. Must match the configured `DIMENSION`.
</ParamField>

<ParamField body="similarity" type="Similarity" required>
  The distance metric to use.
</ParamField>

<ParamField body="limit" type="uint64" required>
  Maximum number of results to return.
</ParamField>

<ParamField body="ef" type="uint64">
  Search breadth for HNSW. Larger values trade speed for accuracy. Defaults to the server's `HNSW_EF` setting.
</ParamField>

**Request:**

```protobuf
message SearchRequest {
  DenseVector query_vector = 1;
  Similarity similarity = 2;
  uint64 limit = 3;
  uint64 ef = 4;
}
```

**Response:**

```protobuf
message SearchResponse {
  repeated PointID result_point_ids = 1;
}
```

**Example:**

```bash
grpcurl -plaintext \
  -H "authorization: secret" \
  -d '{
    "query_vector": {"values": [0.1, 0.2, 0.3, 0.4]},
    "similarity": 3,
    "limit": 5,
    "ef": 200
  }' \
  localhost:50051 vectordb.VectorDB/SearchPoints
```

**Response:**

```json
{
  "resultPointIds": [
    {"id": {"value": "550e8400-e29b-41d4-a716-446655440000"}},
    {"id": {"value": "6ba7b810-9dad-11d1-80b4-00c04fd430c8"}}
  ]
}
```

---

### InsertVectorsBatch

Insert multiple vectors in a single request.

**Request:**

```protobuf
message InsertVectorsBatchRequest {
  repeated InsertVectorRequest vectors = 1;
}
```

**Response:**

```protobuf
message InsertVectorsBatchResponse {
  repeated PointID ids = 1;
}
```

**Example:**

```bash
grpcurl -plaintext \
  -H "authorization: secret" \
  -d '{
    "vectors": [
      {"vector": {"values": [0.1, 0.2, 0.3]}, "payload": {"content_type": 1, "content": "doc one"}},
      {"vector": {"values": [0.4, 0.5, 0.6]}, "payload": {"content_type": 1, "content": "doc two"}}
    ]
  }' \
  localhost:50051 vectordb.VectorDB/InsertVectorsBatch
```

---

### SearchPointsBatch

Search against multiple query vectors in a single request.

**Request:**

```protobuf
message SearchPointsBatchRequest {
  repeated SearchRequest queries = 1;
}
```

**Response:**

```protobuf
message SearchPointsBatchResponse {
  repeated SearchResponse results = 1;
}
```

**Example:**

```bash
grpcurl -plaintext \
  -H "authorization: secret" \
  -d '{
    "queries": [
      {"query_vector": {"values": [0.1, 0.2, 0.3]}, "similarity": 3, "limit": 2},
      {"query_vector": {"values": [0.4, 0.5, 0.6]}, "similarity": 0, "limit": 2}
    ]
  }' \
  localhost:50051 vectordb.VectorDB/SearchPointsBatch
```

---

## Message Types

### UUID

```protobuf
message UUID {
  string value = 1;  // UUID v4 string
}
```

### DenseVector

```protobuf
message DenseVector {
  repeated float values = 1;  // Vector components
}
```

### Point

```protobuf
message Point {
  PointID id = 1;        // Unique identifier
  Payload payload = 2;   // Associated metadata
  DenseVector vector = 3; // Vector values
}
```

### PointID

```protobuf
message PointID {
  UUID id = 1;
}
```

### Payload

```protobuf
message Payload {
  ContentType content_type = 1;  // Type of content
  string content = 2;            // Content string
}
```

---

## Enums

### Similarity

Distance/similarity metric for search operations.

| Value | Name | Description |
|-------|------|-------------|
| `0` | `Euclidean` | L2 distance (straight line) |
| `1` | `Manhattan` | L1 distance (city block) |
| `2` | `Hamming` | Count of differing elements |
| `3` | `Cosine` | Angular distance |

### ContentType

Type of payload content.

| Value | Name | Description |
|-------|------|-------------|
| `0` | `Image` | Image reference or data |
| `1` | `Text` | Text content |

---

## Error Codes

| gRPC Code | Name | Description |
|-----------|------|-------------|
| `0` | `OK` | Success |
| `3` | `INVALID_ARGUMENT` | Invalid request (e.g., wrong dimensions) |
| `5` | `NOT_FOUND` | Point does not exist |
| `13` | `INTERNAL` | Server error |
| `16` | `UNAUTHENTICATED` | Invalid or missing API key |

---

## Client Libraries

### Python

```python
from vortexdb import VortexDB, DenseVector, Payload, Similarity

with VortexDB(grpc_url="localhost:50051", api_key="secret") as db:
    # Insert
    point_id = db.insert(
        vector=DenseVector([0.1, 0.2, 0.3, 0.4]),
        payload=Payload.text("Hello")
    )

    # Batch insert
    ids = db.batch_insert(items=[
        (DenseVector([0.1, 0.2, 0.3]), Payload.text("doc one")),
        (DenseVector([0.4, 0.5, 0.6]), Payload.text("doc two")),
    ])

    # Search with ef parameter
    results = db.search(
        vector=DenseVector([0.1, 0.2, 0.3, 0.4]),
        similarity=Similarity.COSINE,
        limit=5,
        ef=200,
    )
```

### Generate Clients

Use `protoc` to generate clients in any language:

```bash
python -m grpc_tools.protoc \
  -I./crates/grpc/proto \
  --python_out=./client \
  --grpc_python_out=./client \
  ./crates/grpc/proto/vector-db.proto
```

## Next Steps

<CardGroup cols={2}>
  <Card title="HTTP API" icon="globe" href="/api-reference/http">
    REST API reference
  </Card>
  <Card title="Python SDK" icon="python" href="/sdk/reference">
    Python client documentation
  </Card>
</CardGroup>
