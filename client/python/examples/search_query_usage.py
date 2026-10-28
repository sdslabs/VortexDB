from vortexdb import VortexDB
from vortexdb import DenseQuery, PointInput, Similarity, SearchQuery, SparseVector
from vortexdb import to_dense_vectors


def main():
    db = VortexDB(
        grpc_url="localhost:50051",
        api_key="my-secret-password",
    )

    raw_vectors = [
        [0.1, 0.2, 0.3],
        [0.4, 0.5, 0.6],
        [0.7, 0.8, 0.9],
    ]
    vectors = to_dense_vectors(raw_vectors)

    point_ids = db.batch_insert(
        points=[
            PointInput(
                dense_vector=vectors[0], sparse_vector=SparseVector([1, 5], [1.0, 0.5])
            ),
            PointInput(
                dense_vector=vectors[1], sparse_vector=SparseVector([5, 9], [2.0, 1.0])
            ),
            PointInput(dense_vector=vectors[2]),
        ]
    )

    q = SearchQuery(limit=3, query=DenseQuery(vectors[0], Similarity.COSINE))
    res = db.search(query=q)
    print("Dense search:\n", res)

    sparse_q = SearchQuery(limit=3, query=SparseVector([5], [1.0]))
    res = db.search(query=sparse_q)
    print("\nSparse search:\n", res)

    queries = [
        SearchQuery(3, DenseQuery(vectors[0], Similarity.HAMMING)),
        SearchQuery(2, DenseQuery(vectors[1], Similarity.EUCLIDEAN)),
        sparse_q,
    ]
    res = db.batch_search(queries=queries)
    print("\nBatch search:\n", res)

    for pid in point_ids:
        db.delete(point_id=pid)

    db.close()


if __name__ == "__main__":
    main()
