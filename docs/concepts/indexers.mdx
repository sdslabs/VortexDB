---
title: "Indexers"
description: "Choosing the right vector index for your use case"
---

# Indexers

VortexDB supports multiple indexing algorithms, each optimized for different use cases. The index determines how vectors are organized for similarity search.

## Flat Index

The **Flat** index performs brute-force exhaustive search by computing distances to every vector.

### When to Use

- **Dataset size**: < 10,000 vectors
- **Requirements**: Need exact/guaranteed results
- **Use cases**: Testing, prototyping, small production workloads

```bash
INDEX_TYPE=flat
```

## KD-Tree Index

The **KD-Tree** (k-dimensional tree) is a space-partitioning data structure that recursively divides the vector space.

At each level, the tree splits data along a different dimension (cycling through x, y, z, ...).

### When to Use

- **Vector dimensions**: < 20 dimensions
- **Dataset size**: Thousands to hundreds of thousands of vectors
- **Use cases**: Geographic data, low-dimensional embeddings, spatial queries

```bash
INDEX_TYPE=kdtree
```

<Warning>
For high-dimensional embeddings (e.g., 384, 768, 1536 dimensions common in ML), KD-Tree is not recommended. Use HNSW instead.
</Warning>

## HNSW Index

**HNSW** (Hierarchical Navigable Small World) is a state-of-the-art approximate nearest neighbor algorithm based on proximity graphs. It constructs a multi-layered graph where each layer represents a different level of granularity, enabling efficient navigation from coarse to fine-grained similarity search. 

Check out this [blog post](https://blog.sdslabs.co/2026/03/hnsw-index) for more theoretical details and [this blog](https://blog.sdslabs.co/2026/03/hnsw-indexp2) covering implementation in VortexDB.

### When to Use

- **Vector dimensions**: Any, but especially > 20 dimensions
- **Dataset size**: 100,000+ vectors
- **Use cases**: Semantic search, recommendation systems, RAG applications

```bash
INDEX_TYPE=hnsw
```

## Distance Metrics

All indexes support four distance/similarity metrics:

<Tabs>
  <Tab title="Cosine">
    Measures the angle between two vectors, ignoring magnitude.
    
    $$d = 1 - \frac{A \cdot B}{\|A\| \|B\|}$$
    
    - **Range**: 0 (identical) to 2 (opposite)
    - **Best for**: Text embeddings, normalized vectors
    
    ```python
    similarity=Similarity.COSINE
    ```
  </Tab>
  <Tab title="Euclidean">
    L2 distance—the straight-line distance between points.
    
    $$d = \sqrt{\sum_{i=1}^{n} (a_i - b_i)^2}$$
    
    - **Range**: 0 (identical) to ∞
    - **Best for**: General purpose
    
    ```python
    similarity=Similarity.EUCLIDEAN
    ```
  </Tab>
  <Tab title="Manhattan">
    L1 distance—the sum of absolute differences.
    
    $$d = \sum_{i=1}^{n} |a_i - b_i|$$
    
    - **Range**: 0 (identical) to ∞
    - **Best for**: Sparse vectors, grid-based data
    
    ```python
    similarity=Similarity.MANHATTAN
    ```
  </Tab>
  <Tab title="Hamming">
    Counts positions where elements differ.
    
    - **Range**: 0 (identical) to n (completely different)
    - **Best for**: Binary vectors, categorical data
    
    ```python
    similarity=Similarity.HAMMING
    ```
  </Tab>
</Tabs>

## Choosing an Index

Use this decision tree:

| Vectors | Dimensions | Recommended Index |
|---------|------------|-------------------|
| < 10,000 | Any | **Flat** |
| 10K - 100K | < 20 | **KD-Tree** |
| 10K - 100K | ≥ 20 | **HNSW** |
| > 100K | Any | **HNSW** |

## Configuration Example

```bash
# For a semantic search application with OpenAI embeddings
DIMENSION=1536
INDEX_TYPE=hnsw
STORAGE_TYPE=rocksdb

# For a small prototype with sentence-transformers
DIMENSION=384
INDEX_TYPE=flat
STORAGE_TYPE=inmemory
```

## Next Steps

<CardGroup cols={2}>
  <Card title="Snapshots" icon="camera" href="/concepts/snapshots">
    Learn how to backup and restore your index
  </Card>
  <Card title="API Reference" icon="code" href="/api-reference/overview">
    Explore the complete API
  </Card>
</CardGroup>
