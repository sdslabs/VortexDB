## Installation (local / development)

At the moment, the client lives inside the main VortexDB repository.

From `client/python`:

```bash
python -m venv .venv
source .venv/bin/activate

pip install -e .
```

---

## Configuration and Authentication 

The client communicates with VortexDB over gRPC and requires:

- gRPC endpoint (host:port)
- API key (maps to `GRPC_ROOT_PASSWORD` on the server)

These can be provided either:

- explicitly when constructing the client, or 
- via environment variables

---

## Basic Usage

All examples and methods of using the client are in the `examples` directory, in `*_usage.py` files  
A full working example of the basic ways to use the client is available in:   
`examples/basic_usage.py`

### Context Manager Support

The client supports usage as a context manager, which automatically closes the underlying gRPC channel connection  
Example available in:  
```examples/context_manager_usage.py```

### Async Client Support

For async applications, use `AsyncVortexDB`. It mirrors the synchronous client API and uses `grpc.aio` under the hood, including full support for `batch_insert` and `batch_search`.

Examples available in:
```examples/async_usage.py``` & ```examples/async_batch_usage.py```

```python
async with AsyncVortexDB(
    grpc_url="localhost:50051",
    api_key="your-api-key",
) as db:
    point_id = await db.insert(
        vector=DenseVector([0.1, 0.2, 0.3]),
        payload=Payload.text("hello async vortex"),
    )
```

### Batch Insertion and Search Support  

Both `VortexDB` and `AsyncVortexDB` support batch insertion and batch search queries.  
Methods of usage and examples available in:  
```examples/batch_insert_usage.py``` & ```examples/search_query_usage.py``` & ```examples/async_batch_usage.py```  

---

## Client API

### `VortexDB`

Main client class for interacting with the VortexDB gRPC server.

### `AsyncVortexDB`

Async client class for I/O-heavy applications. It has the same constructor and method names as `VortexDB`, but methods are awaitable:

```
await db.insert(...)
await db.batch_insert(...)
await db.get(...)
await db.search(...)
await db.batch_search(...)
await db.delete(...)
await db.close()
```

It also supports async context manager usage:

```
async with AsyncVortexDB(...) as db:
    ...
```

#### **Constructor**

```
VortexDB(
    grpc_url: str | None = None,
    api_key: str | None = None,
    timeout: float | None = None,
)
```
`grpc_url`: gRPC server address (`host:port`)  
`api_key`: API key for authentication  
`timeout`: per-request timeout in seconds  

---

#### **Insert**

Insert a vector with an associated payload  
```
insert(*, vector: DenseVector, payload: Payload) -> str
```

Returns
- `point_id` (UUID string)

Raises
- `TypeError` if `vector` is not a `DenseVector`
- gRPC-mapped errors (see Error Handling)

---

#### **Batch Insert**

Insert multiple vectors with payloads in a single request  
```
batch_insert(*, items: list[tuple[DenseVector, Payload]]) -> list[str]
```

Returns
- List of `point_id` (UUID string)

Raises
- `TypeError` if input structure is invalid
- gRPC-mapped errors (see Error Handling)

---

#### **Get**

Fetch a point by its ID
```
get(*, point_id: str) -> Point | None
```

Returns 
- `Point` if found
- `None` if the point does not exist

---

#### **Search**

Search for nearest neighbours to a query vector
```
search(
    *,
    vector: DenseVector,
    similarity: Similarity,
    limit: int,
) -> list[str]
```

Returns
- List of `point_id` strings

Raises
- `TypeError` if `vector` is not a `DenseVector`
- `InvalidArgumentError` for invalid parameters

---

#### **Batch Search**

Search for nearest neighbours for multiple queries in a single request
```
batch_search(
    *,
    queries,
    similarity: Similarity | None = None,
    limit: int | None = None,
) -> list[list[str]]
```

Returns 
- `TypeError` for invalid query formats
- `ValueError` if required parameters are missing

Supported Input Formats:  
The `queries` parameter is flexible and supports multiple formats:
- List of `SearchQuery` objects
- List of `(DenseVector, Similarity, Limit)` tuples
- List of `(DenseVector, Similarity)` tuples with a global `Limit`
- List of `(DenseVector, Limit)` tuples with a global `Similarity`
- List of `DenseVector` with global `Similarity` and `Limit`

---

#### **Delete**

Delete a point by its ID
```
delete(*, point_id: str) -> None
```

Raises
- `NotFoundError` if the point does not exist

---

#### **Close**

Close the underlying gRPC channel
```
close() -> None
```

---

## Models  

The client exposes typed models that represent VortexDB concepts and handle
validation and protobuf conversion internally

### `DenseVector`

``` 
DenseVector(values: list[float] | tuple[float, ...])
```
- Validates numeric input
- Normalizes values to `float`
- Immutable (`frozen=True`)

--- 

### `Payload`

```
Payload(content_type: ContentType, content:str)
```
Factory Helpers:
- `Payload.text(content: str)`
- `Payload.image(content: str)`

---

### `Point`

```
Point(
    id: str,
    vector: DenseVector,
    payload: Payload,
)
```
Additional `pretty()` method provided to properly format output  
All fields are directly accessible:  
- `point.id`
- `point.vector`
- `point.payload`  

---

### `SearchQuery`

```
SearchQuery(
    vector: DenseVector,
    similarity: Similarity,
    limit: int,
)
```
Structured representation of a search request  

---

### `Similarity`

Enum representing distance functions: 
- `EUCLIDEAN`
- `MANHATTAN`
- `HAMMING`
- `COSINE`

---

### `ContentType`

Enum representing payload type:
- `TEXT`
- `IMAGE`

---

## Error Handling

The client maps gRPC status codes to Python exceptions to provide a clean, Pythonic error-handling experience.  
All client exceptions inherit from `VortexDBError`  

### Exception Mapping

| gRPC Status Code | Python Exception |
| :--- | :--- |
| `UNAUTHENTICATED` | `AuthenticationError` |
| `NOT_FOUND` | `NotFoundError` |
| `INVALID_ARGUMENT` | `InvalidArgumentError` |
| `DEADLINE_EXCEEDED` | `TimeoutError` |
| `UNAVAILABLE` | `ServiceUnavailableError` |
| Any other error | `InternalServerError` |

---

## Testing

Tests are written using **pytest**  
Test coverage includes:  
- models
- configuration loading
- gRPC connection layer
- client API

Tests live in the `tests/` directory  

To run the tests: `pytest -v`

---

## Proto and gRPC Stubs

The gRPC interface is defined using a Protocol Buffers (`.proto`) file, from which Python gRPC stubs are generated. 

### Proto

The `.proto` file is kept here for transparency and reproducibility reasons.  

**Location:** `../../crates/grpc/proto/vector-db.proto`


Even though the gRPC server is already running and exposes these methods, the client still needs the proto to:
- Generate strongly-typed request / response classes
- Generate the gRPC client stub (`VectorDBStub`)

---

### Generated Python stubs

**Location:** `vortexdb/grpc/`
  - `vector_db_pb2.py`
  - `vector_db_pb2_grpc.py`

These files are **auto-generated** from `vector-db.proto` and should not be edited manually.  
The client internally wraps this stub. End users never interact with it directly.  

---

### Regenerating the stubs

Regeneration will be only required if the `.proto` file is changed. For example, if:  
- a new RPC is added
- enums are updated

From the client's top-level directory, run: 

```bash
python -m grpc_tools.protoc \
  -I ../../crates/grpc/proto \
  --python_out=vortexdb/grpc \
  --grpc_python_out=vortexdb/grpc \
  ../../crates/grpc/proto/vector-db.proto
```

After running this:  
- `vector_db_pb2_grpc.py` and `vector_db_pb2.py` will be updated
- No other client code should need changes