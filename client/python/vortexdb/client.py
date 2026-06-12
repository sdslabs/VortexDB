from typing import List, Sequence

from vortexdb.connection import GRPCConnection
from vortexdb.config import VortexDBConfig
from vortexdb.models import (
    DenseVector,
    Payload,
    Point,
    Similarity,
)

from vortexdb import protoutils as proto


class VortexDB:
    """ High-level Python client for VortexDB """

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

    def insert(self, *, vector: DenseVector, payload: Payload) -> str:
        """
        Insert a vector with payload.
        Returns: point_id (str)
        """
        self._validate_dense_vector(vector)

        request = proto.build_insert_request(
            vector=vector,
            payload=payload,
        )

        response = self._conn.call(
            self._conn.stub.InsertVector,
            request,
        )

        return response.id.value

    def insert_batch(
        self,
        *,
        points: Sequence[tuple[DenseVector, Payload]],
    ) -> List[str]:
        """
        Insert multiple vectors with payloads.
        Returns: List of point IDs
        """
        for vector, _ in points:
            self._validate_dense_vector(vector)

        request = proto.build_batch_insert_request(
            points=list(points),
        )

        response = self._conn.call(
            self._conn.stub.InsertVectorsBatch,
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

    def search(
        self,
        *,
        vector: DenseVector,
        similarity: Similarity,
        limit: int,
    ) -> List[str]:
        """
        Search for nearest neighbors.
        Returns: List of point IDs
        """
        self._validate_dense_vector(vector)

        request = proto.build_search_request(
            vector=vector,
            similarity=similarity,
            limit=limit,
        )

        response = self._conn.call(
            self._conn.stub.SearchPoints,
            request,
        )

        return [pid.id.value for pid in response.result_point_ids]

    def search_batch(
        self,
        *,
        queries: Sequence[tuple[DenseVector, Similarity, int]],
    ) -> List[List[str]]:
        """
        Search nearest neighbors for multiple query vectors.
        Returns: List of result point ID lists
        """
        for vector, _, _ in queries:
            self._validate_dense_vector(vector)

        request = proto.build_batch_search_request(
            queries=list(queries),
        )

        response = self._conn.call(
            self._conn.stub.SearchPointsBatch,
            request,
        )

        return [
            [pid.id.value for pid in result.result_point_ids]
            for result in response.results
        ]

    @staticmethod
    def _validate_dense_vector(vector: DenseVector) -> None:
        if not isinstance(vector, DenseVector):
            raise TypeError(
                "vector must be a DenseVector. "
                "Use: DenseVector([1.0, 2.0, 3.0])"
            )

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
