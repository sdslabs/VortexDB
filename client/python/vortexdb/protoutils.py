from vortexdb.grpc import vector_db_pb2
from vortexdb.models import DenseVector, Payload, Similarity


def build_insert_request(
    *,
    vector: DenseVector,
    payload: Payload,
) -> vector_db_pb2.InsertVectorRequest:
    return vector_db_pb2.InsertVectorRequest(
        vector=vector.to_proto(),
        payload=payload.to_proto(),
    )


def build_batch_insert_request(
    *,
    items: list[tuple[DenseVector, Payload]],
) -> vector_db_pb2.InsertVectorsBatchRequest:
    if not isinstance(items, (list, tuple)):
        raise TypeError("Items must be a list of (DenseVector, Payload) tuples")

    if not items:
        raise ValueError("Items cannot be empty")

    requests = []

    for i, pair in enumerate(items):
        if not isinstance(pair, (list, tuple)) or len(pair) != 2:
            raise TypeError(f"items[{i}] must be a tuple of (DenseVector, Payload)")

        vector, payload = pair
        if not isinstance(vector, DenseVector):
            raise TypeError(
                f"items[{i}][0] must be a DenseVectorUse: DenseVector([1.0, 2.0, 3.0])"
            )

        if not isinstance(payload, Payload):
            raise TypeError(f"items[{i}][1] must be Payload")

        requests.append(build_insert_request(vector=vector, payload=payload))

    return vector_db_pb2.InsertVectorsBatchRequest(vectors=requests)


def build_point_id_request(point_id: str) -> vector_db_pb2.PointID:
    return vector_db_pb2.PointID(id=vector_db_pb2.UUID(value=point_id))


def build_search_request(
    *,
    vector: DenseVector,
    similarity: Similarity,
    limit: int,
    ef: int | None = None,
) -> vector_db_pb2.SearchRequest:
    return vector_db_pb2.SearchRequest(
        query_vector=vector.to_proto(),
        similarity=similarity.to_proto(),
        limit=limit,
        ef=ef or 0,
    )


def build_batch_search_request(
    *,
    queries: list[tuple[DenseVector, Similarity, int]],
    ef: int | None = None,
) -> vector_db_pb2.SearchPointsBatchRequest:
    if not isinstance(queries, (list, tuple)):
        raise TypeError(
            "Queries must be a list of (DenseVector, Similarity, Limit (int)) tuples"
        )

    if not queries:
        raise ValueError("Queries cannot be empty")

    requests = []

    for i, trio in enumerate(queries):
        if not isinstance(trio, (list, tuple)) or len(trio) != 3:
            raise TypeError(
                f"queries[{i}] must be a tuple of (DenseVector, Similarity, Limit(int))"
            )

        vector, similarity, limit = trio
        if not isinstance(vector, DenseVector):
            raise TypeError(
                f"queries[{i}][0] must be a DenseVector"
                "Use: DenseVector([1.0, 2.0, 3.0])"
            )
        if not isinstance(similarity, Similarity):
            raise TypeError(f"queries[{i}][1] must be Similarity")
        if not isinstance(limit, int):
            raise TypeError(f"queries[{i}][2] must be an integer value")

        requests.append(
            vector_db_pb2.SearchRequest(
                query_vector=vector.to_proto(),
                similarity=similarity.to_proto(),
                limit=limit,
                ef=ef,
            )
        )

    return vector_db_pb2.SearchPointsBatchRequest(queries=requests)
