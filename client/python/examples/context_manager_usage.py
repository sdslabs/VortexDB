from vortexdb import VortexDB, DenseQuery, DenseVector, Payload, SearchQuery, Similarity


def main():
    with VortexDB(
        grpc_url="localhost:50051",
        api_key="my-secret-password",
    ) as db:
        # Insert a vector
        point_id = db.insert(
            dense_vector=DenseVector([0.1, 0.2, 0.3]),
            payload=Payload.text("hello world"),
        )

        # Get the point
        point = db.get(point_id=point_id)
        print("Fetched point:", point.pretty())

        # Search
        results = db.search(
            query=SearchQuery(
                limit=3,
                query=DenseQuery(DenseVector([0.1, 0.2, 0.3]), Similarity.COSINE),
            )
        )
        print("Search results:", results)

        # Delete
        db.delete(point_id=point_id)

    # At this point, the gRPC channel is closed automatically
    print("Connection closed")


if __name__ == "__main__":
    main()
