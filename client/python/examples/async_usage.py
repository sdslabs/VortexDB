import asyncio

from vortexdb import AsyncVortexDB, DenseVector, Payload, Similarity


async def main():
    async with AsyncVortexDB(
        grpc_url="localhost:50051",
        api_key="my-secret-password",
    ) as db:
        point_id = await db.insert(
            vector=DenseVector([0.1, 0.2, 0.3]),
            payload=Payload.text("hello async vortex"),
        )

        point = await db.get(point_id=point_id)
        if point is not None:
            print(point.pretty())

        results = await db.search(
            vector=DenseVector([0.1, 0.2, 0.3]),
            similarity=Similarity.COSINE,
            limit=5,
        )
        print(results)

        await db.delete(point_id=point_id)


if __name__ == "__main__":
    asyncio.run(main())
