---
title: "SDK Reference"
description: "Complete Python SDK API documentation"
---

# Python SDK Reference

Complete API documentation for the VortexDB Python client.

## Installation

```bash
pip install vortexdb
```

---

## VortexDB

The main client class for interacting with VortexDB.

```python
from vortexdb import VortexDB
```

### Constructor

```python
VortexDB(
    *,
    grpc_url: str | None = None,
    api_key: str | None = None,
    timeout: float | None = None,
)
```

<ParamField body="grpc_url" type="str" default="localhost:50051">
  The gRPC server address. Can also be set via `VORTEXDB_GRPC_URL` environment variable.
</ParamField>

<ParamField body="api_key" type="str" default="None">
  Authentication key for the gRPC API. Can also be set via `VORTEXDB_API_KEY` environment variable.
</ParamField>

<ParamField body="timeout" type="float" default="30.0">
  Request timeout in seconds. Can also be set via `VORTEXDB_TIMEOUT` environment variable.
</ParamField>

**Example:**

```python
# Explicit configuration
db = VortexDB(
    grpc_url="localhost:50051",
    api_key="secret",
    timeout=60.0,
)

# Using environment variables
import os
os.environ["VORTEXDB_GRPC_URL"] = "localhost:50051"
os.environ["VORTEXDB_API_KEY"] = "secret"
db = VortexDB()
```

---

### Methods

#### insert

Insert a vector with its payload into the database.

```python
def insert(
    self,
    *,
    vector: DenseVector,
    payload: Payload,
) -> str
```

<ResponseField name="return" type="str">
  UUID of the created point.
</ResponseField>

**Example:**

```python
point_id = db.insert(
    vector=DenseVector([0.1, 0.2, 0.3, 0.4]),
    payload=Payload.text("My document"),
)
```

---

#### batch_insert

Insert multiple vectors in a single request.

```python
def batch_insert(
    self,
    *,
    items: list[tuple[DenseVector, Payload]],
) -> list[str]
```

<ResponseField name="return" type="list[str]">
  List of UUIDs for the created points, in input order.
</ResponseField>

**Example:**

```python
ids = db.batch_insert(items=[
    (DenseVector([0.1, 0.2, 0.3]), Payload.text("doc one")),
    (DenseVector([0.4, 0.5, 0.6]), Payload.text("doc two")),
])
```

---

#### get

Retrieve a point by its ID.

```python
def get(
    self,
    *,
    point_id: str,
) -> Point | None
```

<ResponseField name="return" type="Point | None">
  The point if found, `None` otherwise.
</ResponseField>

**Example:**

```python
point = db.get(point_id="550e8400-e29b-41d4-a716-446655440000")
if point:
    print(f"Vector: {point.vector.to_list()}")
    print(f"Payload: {point.payload.content}")
```

---

#### search

Search for the k nearest neighbors to a query vector.

```python
def search(
    self,
    *,
    vector: DenseVector | None = None,
    similarity: Similarity | None = None,
    limit: int | None = None,
    query: SearchQuery | None = None,
    ef: int | None = None,
) -> List[str]
```

<ParamField body="query" type="SearchQuery" default="None">
  A `SearchQuery` object bundling vector, similarity, and limit. Use this or pass individual args.
</ParamField>

<ParamField body="ef" type="int" default="None">
  Search breadth for HNSW. Uses server default if not set.
</ParamField>

<ResponseField name="return" type="List[str]">
  List of point IDs ordered by similarity (closest first).
</ResponseField>

**Example:**

```python
# Using a SearchQuery
query = SearchQuery(DenseVector([0.1, 0.2, 0.3, 0.4]), Similarity.COSINE, 10)
results = db.search(query=query)

# Using individual args with ef
results = db.search(
    vector=DenseVector([0.1, 0.2, 0.3, 0.4]),
    similarity=Similarity.COSINE,
    limit=10,
    ef=200,
)
```

---

#### batch_search

Search against multiple query vectors in a single request.

```python
def batch_search(
    self,
    *,
    queries,
    similarity: Similarity | None = None,
    limit: int | None = None,
    ef: int | None = None,
) -> List[List[str]]
```

Accepts `List[SearchQuery]`, `List[(DenseVector, Similarity, int)]`, or bare `List[DenseVector]` with global `similarity` and `limit`.

<ResponseField name="return" type="List[List[str]]">
  One result list per input query.
</ResponseField>

**Example:**

```python
results = db.batch_search(queries=[
    SearchQuery(DenseVector([0.1, 0.2, 0.3]), Similarity.COSINE, 5),
    (DenseVector([0.4, 0.5, 0.6]), Similarity.EUCLIDEAN, 3),
])
```

---

#### delete

Delete a point by its ID.

```python
def delete(
    self,
    *,
    point_id: str,
) -> None
```

**Example:**

```python
db.delete(point_id="550e8400-e29b-41d4-a716-446655440000")
```

---

#### close

Close the gRPC connection.

```python
def close(self) -> None
```

**Example:**

```python
db = VortexDB(grpc_url="localhost:50051", api_key="secret")
# ... use the client ...
db.close()
```

---

### Context Manager

The client supports the context manager protocol for automatic cleanup:

```python
with VortexDB(grpc_url="localhost:50051", api_key="secret") as db:
    point_id = db.insert(
        vector=DenseVector([0.1, 0.2, 0.3]),
        payload=Payload.text("Hello"),
    )
# Connection automatically closed
```

---

## DenseVector

An immutable dense vector of floating-point values.

```python
from vortexdb import DenseVector
```

### Constructor

```python
DenseVector(values: List[float] | Tuple[float, ...])
```

<ParamField body="values" type="List[float] | Tuple[float, ...]" required>
  The vector components. Must be non-empty and contain numeric values.
</ParamField>

**Raises:**
- `TypeError`: If values is not a list or tuple
- `ValueError`: If values is empty
- `TypeError`: If any value is not numeric

**Example:**

```python
# From list
vec = DenseVector([0.1, 0.2, 0.3, 0.4])

# From tuple
vec = DenseVector((0.1, 0.2, 0.3, 0.4))

# Integers are converted to floats
vec = DenseVector([1, 2, 3, 4])  # -> [1.0, 2.0, 3.0, 4.0]
```

### Methods

#### to_list

Convert the vector to a Python list.

```python
def to_list(self) -> list[float]
```

**Example:**

```python
vec = DenseVector([0.1, 0.2, 0.3])
values = vec.to_list()  # [0.1, 0.2, 0.3]
```

### Properties

#### values

Access the vector values (read-only).

```python
vec = DenseVector([0.1, 0.2, 0.3])
print(vec.values)  # (0.1, 0.2, 0.3)
```

---

## Payload

Metadata associated with a vector.

```python
from vortexdb import Payload
```

### Factory Methods

#### text

Create a text payload.

```python
@staticmethod
def text(content: str) -> Payload
```

**Example:**

```python
payload = Payload.text("This is my document content")
```

#### image

Create an image payload.

```python
@staticmethod
def image(content: str) -> Payload
```

**Example:**

```python
payload = Payload.image("path/to/image.jpg")
```

### Constructor

```python
Payload(content_type: ContentType, content: str)
```

<ParamField body="content_type" type="ContentType" required>
  The type of content (`ContentType.TEXT` or `ContentType.IMAGE`).
</ParamField>

<ParamField body="content" type="str" required>
  The content string.
</ParamField>

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `content_type` | `ContentType` | Type of payload |
| `content` | `str` | Content string |

---

## Point

A point returned from the database (vector + payload + ID).

```python
from vortexdb.models import Point
```

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Point UUID |
| `vector` | `DenseVector` | The vector values |
| `payload` | `Payload` | Associated metadata |

### Methods

#### pretty

Return a formatted string representation.

```python
def pretty(self) -> str
```

**Example:**

```python
point = db.get(point_id="...")
print(point.pretty())
# Output:
# Point ID: 550e8400-e29b-41d4-a716-446655440000
# Vector: [0.1, 0.2, 0.3, 0.4]
# Payload Type: Text
# Payload Content: My document
```

---

## Similarity

Enum for distance/similarity metrics.

```python
from vortexdb import Similarity
```

### Values

| Value | Description |
|-------|-------------|
| `Similarity.EUCLIDEAN` | L2 distance (straight line) |
| `Similarity.MANHATTAN` | L1 distance (city block) |
| `Similarity.HAMMING` | Count of differing elements |
| `Similarity.COSINE` | Angular distance |

**Example:**

```python
from vortexdb import Similarity

# Use in search
results = db.search(
    vector=DenseVector([0.1, 0.2, 0.3]),
    similarity=Similarity.COSINE,
    limit=5,
)
```

---

## ContentType

Enum for payload content types.

```python
from vortexdb.models import ContentType
```

### Values

| Value | Description |
|-------|-------------|
| `ContentType.TEXT` | Text content |
| `ContentType.IMAGE` | Image reference |

---

## SearchQuery

Bundles a search's parameters into a single object.

```python
from vortexdb import SearchQuery

query = SearchQuery(DenseVector([0.1, 0.2, 0.3]), Similarity.COSINE, 10)
results = db.search(query=query)
```

| Param | Type | Description |
|-------|------|-------------|
| `vector` | `DenseVector` | Query vector |
| `similarity` | `Similarity` | Distance metric |
| `limit` | `int` | Max results |

---

## Exceptions

All exceptions inherit from `VortexDBError`.

```python
from vortexdb import (
    VortexDBError,
    AuthenticationError,
    NotFoundError,
    InvalidArgumentError,
    TimeoutError,
    ServiceUnavailableError,
    InternalServerError,
)
```

### Exception Hierarchy

| Exception | Description |
|-----------|-------------|
| `VortexDBError` | Base exception for all errors |
| `AuthenticationError` | Invalid or missing API key |
| `NotFoundError` | Requested resource not found |
| `InvalidArgumentError` | Invalid input parameters |
| `TimeoutError` | Request timed out |
| `ServiceUnavailableError` | Server is unavailable |
| `InternalServerError` | Server-side error |

**Example:**

```python
from vortexdb.exceptions import (
    AuthenticationError,
    NotFoundError,
    VortexDBError,
)

try:
    point = db.get(point_id="nonexistent")
except NotFoundError:
    print("Point does not exist")
except AuthenticationError:
    print("Check your API key")
except VortexDBError as e:
    print(f"Unexpected error: {e}")
```

---

## Configuration

### VortexDBConfig

Internal configuration class (usually not used directly).

```python
from vortexdb.config import VortexDBConfig

config = VortexDBConfig.from_env(
    grpc_url="localhost:50051",
    api_key="secret",
    timeout=30.0,
)
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `VORTEXDB_GRPC_URL` | Server address | `localhost:50051` |
| `VORTEXDB_API_KEY` | Authentication key | None |
| `VORTEXDB_TIMEOUT` | Request timeout (seconds) | `30.0` |

---

## Type Hints

The SDK is fully typed. Import types for type hints:

```python
from typing import List, Optional
from vortexdb import VortexDB, DenseVector, Payload, Similarity
from vortexdb.models import Point, ContentType

def search_documents(
    db: VortexDB,
    query_vector: List[float],
    limit: int = 10,
) -> List[Optional[Point]]:
    results = db.search(
        vector=DenseVector(query_vector),
        similarity=Similarity.COSINE,
        limit=limit,
    )
    return [db.get(point_id=pid) for pid in results]
```

## Next Steps

<CardGroup cols={2}>
  <Card title="SDK Examples" icon="code" href="/sdk/examples">
    Working code examples
  </Card>
  <Card title="API Reference" icon="code" href="/api-reference/overview">
    gRPC and HTTP API docs
  </Card>
</CardGroup>
