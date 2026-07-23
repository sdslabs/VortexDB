---
title: "SDK Examples"
description: "Working code examples for the VortexDB Python SDK"
---

# SDK Examples

Ready-to-run code examples demonstrating VortexDB Python SDK usage.

## Basic Usage

```python
from vortexdb import VortexDB, DenseVector, Payload, Similarity

with VortexDB(grpc_url="localhost:50051", api_key="secret") as db:
    # Insert
    point_id = db.insert(
        vector=DenseVector([0.1, 0.2, 0.3]),
        payload=Payload.text("hello world"),
    )
    print(f"Inserted: {point_id}")

    # Batch insert
    ids = db.batch_insert(items=[
        (DenseVector([0.1, 0.2, 0.3]), Payload.text("doc one")),
        (DenseVector([0.4, 0.5, 0.6]), Payload.text("doc two")),
    ])

    # Search
    results = db.search(
        vector=DenseVector([0.1, 0.2, 0.3]),
        similarity=Similarity.COSINE,
        limit=3,
    )
    print(f"Found {len(results)} results")
```

---

## Semantic Search

Using sentence-transformers for text embedding:

```python
from vortexdb import VortexDB, DenseVector, Payload, Similarity
from sentence_transformers import SentenceTransformer

model = SentenceTransformer('all-MiniLM-L6-v2')

documents = [
    "The quick brown fox jumps over the lazy dog",
    "Machine learning is a subset of artificial intelligence",
    "Python is a popular programming language",
]

def embed(text: str) -> DenseVector:
    return DenseVector(model.encode(text).tolist())

with VortexDB(grpc_url="localhost:50051", api_key="secret") as db:
    for doc in documents:
        db.insert(vector=embed(doc), payload=Payload.text(doc))

    results = db.search(
        vector=embed("AI and programming"),
        similarity=Similarity.COSINE,
        limit=2,
    )
    for pid in results:
        point = db.get(point_id=pid)
        print(f"  {point.payload.content}")
```

---

## Batch Processing

```python
from vortexdb import VortexDB, DenseVector, Payload, Similarity

with VortexDB(grpc_url="localhost:50051", api_key="secret") as db:
    items = [(DenseVector([i * 0.1 for _ in range(3)]), Payload.text(f"doc {i}")) for i in range(100)]
    ids = db.batch_insert(items=items)

    # Batch search
    queries = [
        (DenseVector([0.1, 0.2, 0.3]), Similarity.COSINE, 3),
        (DenseVector([0.4, 0.5, 0.6]), Similarity.EUCLIDEAN, 3),
    ]
    batch_results = db.batch_search(queries=queries)
    for i, res in enumerate(batch_results):
        print(f"Query {i}: {len(res)} results")
```

---

## Testing with pytest

```python
import pytest
from vortexdb import VortexDB, DenseVector, Payload, Similarity

@pytest.fixture
def db():
    client = VortexDB(grpc_url="localhost:50051", api_key="secret")
    yield client
    client.close()

class TestVortexDB:
    def test_insert_and_get(self, db):
        point_id = db.insert(
            vector=DenseVector([0.1, 0.2, 0.3, 0.4]),
            payload=Payload.text("Test document"),
        )
        point = db.get(point_id=point_id)
        assert point is not None
        assert point.payload.content == "Test document"
        db.delete(point_id=point_id)

    def test_search(self, db):
        point_id = db.insert(
            vector=DenseVector([1.0, 2.0, 3.0]),
            payload=Payload.text("target"),
        )
        results = db.search(
            vector=DenseVector([1.0, 2.0, 3.0]),
            similarity=Similarity.COSINE,
            limit=10,
        )
        assert point_id in results
        db.delete(point_id=point_id)
```

---

## Next Steps

<CardGroup cols={2}>
  <Card title="SDK Reference" icon="book" href="/sdk/reference">
    Complete API documentation
  </Card>
  <Card title="API Reference" icon="code" href="/api-reference/overview">
    gRPC and HTTP API docs
  </Card>
</CardGroup>
