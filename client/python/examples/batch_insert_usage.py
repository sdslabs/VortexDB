from vortexdb import VortexDB
from vortexdb import Payload, PointInput, SparseVector, to_dense_vectors


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

    points = [
        PointInput(dense_vector=vectors[0], payload=Payload.text("hello world")),
        PointInput(
            sparse_vector=SparseVector([3, 17, 42], [0.5, 1.2, 0.8]),
            payload=Payload.image("/img/a.png"),
        ),
        PointInput(
            dense_vector=vectors[2],
            sparse_vector=SparseVector([17], [2.0]),
            payload=Payload.text("foo bar"),
        ),
    ]

    point_ids = db.batch_insert(points=points)
    print("Inserted ids:\n", point_ids)

    for pid in point_ids:
        db.delete(point_id=pid)

    db.close()


if __name__ == "__main__":
    main()
