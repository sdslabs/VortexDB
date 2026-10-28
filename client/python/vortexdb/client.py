from vortexdb.connection import GRPCConnection
from vortexdb.config import VortexDBConfig
from vortexdb.models import (
    DenseVector,
    Payload,
    Point,
    PointInput,
    SearchHit,
    SearchQuery,
    SparseVector,
)

from vortexdb import protoutils as proto


class VortexDB:
    """High-level Python client for VortexDB"""

    def __init__(
        self,
        *,
        grpc_url: str | None = None,
        api_key: str | None = None,
        timeout: float | None = None,
    ):
        # Config order followed - args -> env vars -> defaults
        self._config = VortexDBConfig.from_env(
            grpc_url=grpc_url,
            api_key=api_key,
            timeout=timeout,
        )

        self._conn = GRPCConnection(self._config)

    # The basic operations

    def insert(
        self,
        *,
        dense_vector: DenseVector | None = None,
        sparse_vector: SparseVector | None = None,
        payload: Payload | None = None,
        point_id: str | None = None,
    ) -> str:
        """
        Insert a point with a dense vector, a sparse vector or both.
        Returns: point_id (str)
        """
        request = proto.build_insert_request(
            PointInput(
                dense_vector=dense_vector,
                sparse_vector=sparse_vector,
                payload=payload,
                point_id=point_id,
            )
        )

        response = self._conn.call(
            self._conn.stub.InsertPoint,
            request,
        )

        return response.id.value

    def batch_insert(self, *, points: list[PointInput]) -> list[str]:
        """
        Insert multiple points.
        Returns: list of point_id (str)
        """
        request = proto.build_batch_insert_request(points=points)

        response = self._conn.call(
            self._conn.stub.InsertPointsBatch,
            request,
        )
        return [pid.id.value for pid in response.ids]

    def get(self, *, point_id: str) -> Point | None:
        """
        Retrieve a point by ID.
        """
        request = proto.build_point_id_request(point_id)

        response = self._conn.call(
            self._conn.stub.GetPoint,
            request,
        )

        if response is None:
            return None

        return Point.from_proto(response)

    def delete(self, *, point_id: str) -> None:
        """
        Delete a point by ID.
        """
        request = proto.build_point_id_request(point_id)

        self._conn.call(
            self._conn.stub.DeletePoint,
            request,
        )

    def search(self, *, query: SearchQuery) -> list[SearchHit]:
        """
        Search with a dense or sparse query.
        Returns: list of SearchHit (dense scores are distances, lower is better;
        sparse scores are dot products, higher is better)
        """
        request = proto.build_search_request(query)

        response = self._conn.call(
            self._conn.stub.Search,
            request,
        )
        return proto.parse_search_response(response)

    def batch_search(self, *, queries: list[SearchQuery]) -> list[list[SearchHit]]:
        """
        Run multiple dense and/or sparse queries in one call.
        Returns: one list of SearchHit per query, in order
        """
        request = proto.build_batch_search_request(queries=queries)

        response = self._conn.call(
            self._conn.stub.SearchBatch,
            request,
        )
        return [proto.parse_search_response(result) for result in response.results]

    def close(self) -> None:
        """
        Close the gRPC connection.
        """
        self._conn.close()

    # Context Manager
    # Will allow the usage of VortexDB with the 'with' keyword (Example given in examples/context_manager_usage.py)
    def __enter__(self) -> "VortexDB":
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self.close()
