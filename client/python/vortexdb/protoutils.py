from vortexdb.grpc import vector_db_pb2
from vortexdb.models import PointInput, SearchHit, SearchQuery


def build_insert_request(point: PointInput) -> vector_db_pb2.PointInput:
    if not isinstance(point, PointInput):
        raise TypeError("point must be a PointInput")

    return point.to_proto()


def build_batch_insert_request(
    *,
    points: list[PointInput],
) -> vector_db_pb2.InsertPointsBatchRequest:
    if not isinstance(points, (list, tuple)):
        raise TypeError("points must be a list of PointInput")

    if not points:
        raise ValueError("points cannot be empty")

    for i, point in enumerate(points):
        if not isinstance(point, PointInput):
            raise TypeError(f"points[{i}] must be a PointInput")

    return vector_db_pb2.InsertPointsBatchRequest(
        points=[point.to_proto() for point in points]
    )


def build_point_id_request(point_id: str) -> vector_db_pb2.PointID:
    return vector_db_pb2.PointID(id=vector_db_pb2.UUID(value=point_id))


def build_search_request(query: SearchQuery) -> vector_db_pb2.SearchRequest:
    if not isinstance(query, SearchQuery):
        raise TypeError("query must be a SearchQuery")

    return query.to_proto()


def build_batch_search_request(
    *,
    queries: list[SearchQuery],
) -> vector_db_pb2.SearchBatchRequest:
    if not isinstance(queries, (list, tuple)):
        raise TypeError("queries must be a list of SearchQuery")

    if not queries:
        raise ValueError("queries cannot be empty")

    for i, query in enumerate(queries):
        if not isinstance(query, SearchQuery):
            raise TypeError(f"queries[{i}] must be a SearchQuery")

    return vector_db_pb2.SearchBatchRequest(
        queries=[query.to_proto() for query in queries]
    )


def parse_search_response(response: vector_db_pb2.SearchResponse) -> list[SearchHit]:
    return [SearchHit.from_proto(hit) for hit in response.hits]
