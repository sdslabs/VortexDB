from vortexdb import VortexDB
from vortexdb import Payload, to_dense_vectors


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

    p1 = Payload.text("hello world")
    p2 = Payload.image("/img/a.png")
    p3 = Payload.text("foo bar")

    items = [
        (vectors[0], p1),
        (vectors[1], p2),
        (vectors[2], p3),
    ]

    # Batch Insert
    point_ids = db.batch_insert(items=items)
    print("Inserted ids:\n", point_ids)

    for pid in point_ids:
        db.delete(point_id=pid)

    db.close()


if __name__ == "__main__":
    main()
