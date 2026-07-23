from vortexdb import VortexDB, DenseVector, Payload, Similarity


def main():
    with VortexDB(
        grpc_url="localhost:50051",
        api_key="my-secret-password",
    ) as db:
        # Insert a vector
        point_id = db.insert(
            vector=DenseVector([0.1, 0.2, 0.3]),
            payload=Payload.text("hello world"),
        )

        # Get the point
        point = db.get(point_id=point_id)
        print("Fetched point:", point.pretty())

        # Search
        results = db.search(
            vector=DenseVector([0.1, 0.2, 0.3]),
            similarity=Similarity.COSINE,
            limit=3,
        )
        print("Search results:", results)

        # Delete
        db.delete(point_id=point_id)

    # At this point, the gRPC channel is closed automatically
    print("Connection closed")


if __name__ == "__main__":
    main()
