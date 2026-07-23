from vortexdb import VortexDB
from vortexdb import DenseVector, Payload, Similarity  # from vortexdb.models


def main():
    # Initialize client
    db = VortexDB(
        grpc_url="localhost:50051",
        api_key="my-secret-password",
    )

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

    # Close connection
    db.close()


if __name__ == "__main__":
    main()
