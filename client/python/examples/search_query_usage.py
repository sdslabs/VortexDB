from vortexdb import VortexDB
from vortexdb import DenseVector, Similarity, SearchQuery, to_dense_vectors


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

    q = SearchQuery(
        vector=vectors[0],
        similarity=Similarity.COSINE,
        limit=3,
    )
    res = db.search(query=q)
    print("Single SearchQuery:\n", res)

    # List of SearchQuery
    queries = [
        SearchQuery(vectors[0], Similarity.HAMMING, 3),
        SearchQuery(vectors[1], Similarity.EUCLIDEAN, 2),
        q,
    ]
    res = db.batch_search(queries=queries)
    print("\nBatch SearchQuery:\n", res)

    # List of vectors with global Similarity and Limit
    res = db.batch_search(
        queries=vectors,
        similarity=Similarity.COSINE,
        limit=3,
    )
    print("\nList of DenseVectors:\n", res)

    # List of tuple (DenseVector, Similarity) with global Limit
    queries = [
        (vectors[0], Similarity.COSINE),
        (vectors[1], Similarity.MANHATTAN),
    ]
    res = db.batch_search(
        queries=queries,
        limit=3,
    )
    print("\nList of (DenseVector, Similarity):\n", res)

    # List of tuple (DenseVector, Limit) with global Similarity
    queries = [
        (vectors[0], 2),
        (vectors[1], 4),
    ]
    res = db.batch_search(
        queries=queries,
        similarity=Similarity.COSINE,
    )
    print("\nList of (DenseVector, Limit):\n", res)

    db.close()


if __name__ == "__main__":
    main()