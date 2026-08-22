---
title: "HTTP API"
description: "RESTful JSON API reference"
---

# HTTP API Reference

VortexDB's HTTP API provides a RESTful interface for vector operations using JSON over HTTP/1.1.

## Base URL

```
http://localhost:3000
```

The port can be configured via the `HTTP_PORT` environment variable. The server binds to `127.0.0.1` by default.

<Warning>
The HTTP API has no authentication. Always deploy behind a reverse proxy or disable it with `DISABLE_HTTP=true` in production.
</Warning>

---

## Endpoints

### Health Check

Check if the server is running.

<ResponseField name="Response" type="string">
  Returns "OK" if the server is healthy.
</ResponseField>

<CodeGroup>
```bash Request
curl http://localhost:3000/health
```

```text Response
OK
```
</CodeGroup>

---

### Root

Verify the server is running.

<CodeGroup>
```bash Request
curl http://localhost:3000/
```

```text Response
Vector Database server is running!
```
</CodeGroup>

---

### Insert Point

<ParamField body="vector" type="array" required>
  Array of floating-point numbers representing the vector. Must match the configured `DIMENSION`.
</ParamField>

<ParamField body="payload" type="object" required>
  Metadata object associated with the vector.

  <Expandable title="payload properties">
    <ParamField body="content_type" type="string" required>
      Type of content: `"Text"` or `"Image"`
    </ParamField>
    <ParamField body="content" type="string" required>
      The content string
    </ParamField>
  </Expandable>
</ParamField>

<ResponseField name="point_id" type="string">
  UUID of the created point.
</ResponseField>

<CodeGroup>
```bash Request
curl -X POST http://localhost:3000/points \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, 0.4],
    "payload": {
      "content_type": "Text",
      "content": "Hello, VortexDB!"
    }
  }'
```

```json Response (201 Created)
{
  "point_id": "550e8400-e29b-41d4-a716-446655440000"
}
```
</CodeGroup>

**Error Responses:**

| Status | Description |
|--------|-------------|
| `400 Bad Request` | Invalid JSON or missing fields |
| `500 Internal Server Error` | Server error during insertion |

---

### Batch Insert

Insert multiple vectors in a single request.

<ParamField body="vectors" type="array" required>
  Array of insert objects, each with `vector` and `payload` fields (same shape as single insert).
</ParamField>

<ResponseField name="point_ids" type="array">
  Array of UUIDs for each created point, in the same order as the input.
</ResponseField>

<CodeGroup>
```bash Request
curl -X POST http://localhost:3000/points/batch \
  -H "Content-Type: application/json" \
  -d '{
    "vectors": [
      {
        "vector": [0.1, 0.2, 0.3, 0.4],
        "payload": {"content_type": "Text", "content": "Document one"}
      },
      {
        "vector": [0.5, 0.6, 0.7, 0.8],
        "payload": {"content_type": "Text", "content": "Document two"}
      }
    ]
  }'
```

```json Response (201 Created)
{
  "point_ids": [
    "550e8400-e29b-41d4-a716-446655440000",
    "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
  ]
}
```
</CodeGroup>

**Error Responses:**

| Status | Description |
|--------|-------------|
| `400 Bad Request` | Invalid JSON or missing fields |
| `500 Internal Server Error` | Server error during insertion |

---

### Get Point

Retrieve a point by its ID.

<ParamField path="id" type="string" required>
  UUID of the point to retrieve.
</ParamField>

<ResponseField name="id" type="string">
  The point's UUID.
</ResponseField>

<ResponseField name="vector" type="array">
  The stored vector values.
</ResponseField>

<ResponseField name="payload" type="object">
  The associated payload metadata.
</ResponseField>

<CodeGroup>
```bash Request
curl http://localhost:3000/points/550e8400-e29b-41d4-a716-446655440000
```

```json Response (200 OK)
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "vector": [0.1, 0.2, 0.3, 0.4],
  "payload": {
    "content_type": "Text",
    "content": "Hello, VortexDB!"
  }
}
```
</CodeGroup>

**Error Responses:**

| Status | Description |
|--------|-------------|
| `404 Not Found` | Point does not exist |
| `500 Internal Server Error` | Server error during retrieval |

---

### Delete Point

Delete a point by its ID.

<ParamField path="id" type="string" required>
  UUID of the point to delete.
</ParamField>

<CodeGroup>
```bash Request
curl -X DELETE http://localhost:3000/points/550e8400-e29b-41d4-a716-446655440000
```

```text Response (204 No Content)
(empty body)
```
</CodeGroup>

**Error Responses:**

| Status | Description |
|--------|-------------|
| `500 Internal Server Error` | Server error during deletion |

---

### Search Points

Search for the k nearest neighbors to a query vector.

<ParamField body="vector" type="array" required>
  Query vector. Must match the configured `DIMENSION`.
</ParamField>

<ParamField body="similarity" type="string" required>
  Distance metric: `"Euclidean"`, `"Manhattan"`, `"Hamming"`, or `"Cosine"`
</ParamField>

<ParamField body="limit" type="integer" required>
  Maximum number of results to return.
</ParamField>

<ResponseField name="results" type="array">
  Array of point IDs ordered by similarity (closest first).
</ResponseField>

<CodeGroup>
```bash Request
curl -X POST http://localhost:3000/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, 0.4],
    "similarity": "Cosine",
    "limit": 5
  }'
```

```json Response (200 OK)
{
  "results": [
    "550e8400-e29b-41d4-a716-446655440000",
    "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "f47ac10b-58cc-4372-a567-0e02b2c3d479"
  ]
}
```
</CodeGroup>

**Error Responses:**

| Status | Description |
|--------|-------------|
| `400 Bad Request` | Invalid JSON or missing fields |
| `500 Internal Server Error` | Server error during search |

---

### Batch Search

Search against multiple query vectors in a single request.

<ParamField body="queries" type="array" required>
  Array of search query objects, each with `vector`, `similarity`, and `limit`.
</ParamField>

<ResponseField name="results" type="array">
  Array of result arrays, one per input query, each ordered by similarity.
</ResponseField>

<CodeGroup>
```bash Request
curl -X POST http://localhost:3000/points/search/batch \
  -H "Content-Type: application/json" \
  -d '{
    "queries": [
      {"vector": [0.1, 0.2, 0.3, 0.4], "similarity": "Cosine", "limit": 2},
      {"vector": [0.5, 0.6, 0.7, 0.8], "similarity": "Euclidean", "limit": 2}
    ]
  }'
```

```json Response (200 OK)
{
  "results": [
    ["550e8400-...", "6ba7b810-..."],
    ["f47ac10b-...", "9a1b2c3d-..."]
  ]
}
```
</CodeGroup>

**Error Responses:**

| Status | Description |
|--------|-------------|
| `400 Bad Request` | Invalid JSON or missing fields |
| `500 Internal Server Error` | Server error during search |

---

## Error Format

Errors are returned as plain text with an appropriate HTTP status code:

```bash
curl -v http://localhost:3000/points/nonexistent-id
```

```
< HTTP/1.1 404 Not Found
< content-type: text/plain; charset=utf-8
<
Point not found
```

---

## Examples

### Complete Workflow

```bash
# 1. Check health
curl http://localhost:3000/health
# OK

# 2. Insert a vector
POINT_ID=$(curl -s -X POST http://localhost:3000/points \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, 0.4],
    "payload": {"content_type": "Text", "content": "First document"}
  }' | jq -r '.point_id')

echo "Created point: $POINT_ID"

# 3. Get the point
curl http://localhost:3000/points/$POINT_ID

# 4. Search for similar vectors
curl -X POST http://localhost:3000/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.15, 0.25, 0.35, 0.45],
    "similarity": "Cosine",
    "limit": 10
  }'

# 5. Delete the point
curl -X DELETE http://localhost:3000/points/$POINT_ID
```

---

## OpenAPI Specification

The complete OpenAPI specification is available at:

```
/docs/openapi.yaml
```

You can import this into tools like Postman or Swagger UI for interactive API exploration.

---

## Next Steps

<CardGroup cols={2}>
  <Card title="gRPC API" icon="bolt" href="/api-reference/grpc">
    High-performance gRPC API reference
  </Card>
  <Card title="Python SDK" icon="python" href="/sdk/reference">
    Python client documentation
  </Card>
</CardGroup>
