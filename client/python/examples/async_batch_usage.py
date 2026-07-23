import asyncio

from vortexdb import AsyncVortexDB
from vortexdb import Payload, Similarity, SearchQuery, to_dense_vectors


async def main():
    async with AsyncVortexDB(
        grpc_url="localhost:50051",
        api_key="my-secret-password",
    ) as db:
        raw_vectors = [
            [0.1, 0.2, 0.3],
            [0.4, 0.5, 0.6],
            [0.7, 0.8, 0.9],
        ]
        vectors = to_dense_vectors(raw_vectors)

        p1 = Payload.text("hello world")
        p2 = Payload.image("/img/a.png")
        p3 = Payload.text("foo bar")

        items = [
            (vectors[0], p1),
            (vectors[1], p2),
            (vectors[2], p3),
        ]

        # Batch Insert
        point_ids = await db.batch_insert(items=items)
        print("Inserted ids:\n", point_ids)

        q = SearchQuery(
            vector=vectors[0],
            similarity=Similarity.COSINE,
            limit=3,
        )
        res = await db.search(query=q)
        print("\nSingle SearchQuery:\n", res)

        # List of SearchQuery
        queries = [
            SearchQuery(vectors[0], Similarity.HAMMING, 3),
            SearchQuery(vectors[1], Similarity.EUCLIDEAN, 2),
            q,
        ]
        res = await db.batch_search(queries=queries)
        print("\nBatch SearchQuery:\n", res)

        # List of vectors with global Similarity and Limit
        res = await db.batch_search(
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
        res = await db.batch_search(
            queries=queries,
            limit=3,
        )
        print("\nList of (DenseVector, Similarity):\n", res)

        # List of tuple (DenseVector, Limit) with global Similarity
        queries = [
            (vectors[0], 2),
            (vectors[1], 4),
        ]
        res = await db.batch_search(
            queries=queries,
            similarity=Similarity.COSINE,
        )
        print("\nList of (DenseVector, Limit):\n", res)

        for pid in point_ids:
            await db.delete(point_id=pid)


if __name__ == "__main__":
    asyncio.run(main())
