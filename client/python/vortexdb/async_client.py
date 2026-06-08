from typing import List

from vortexdb import protoutils as proto
from vortexdb.async_connection import AsyncGRPCConnection
from vortexdb.config import VortexDBConfig
from vortexdb.models import DenseVector, Payload, Point, Similarity


class AsyncVortexDB:
    """High-level async Python client for VortexDB."""

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

        self._conn = AsyncGRPCConnection(self._config)

    async def insert(self, *, vector: DenseVector, payload: Payload) -> str:
        """
        Insert a vector with payload.
        Returns: point_id (str)
        """
        if not isinstance(vector, DenseVector):
            raise TypeError(
                "vector must be a DenseVector. "
                "Use: DenseVector([1.0, 2.0, 3.0])"
            )

        request = proto.build_insert_request(
            vector=vector,
            payload=payload,
        )

        response = await self._conn.call(
            self._conn.stub.InsertVector,
            request,
        )

        return response.id.value

    async def get(self, *, point_id: str) -> Point | None:
        """
        Retrieve a point by ID.
        """
        request = proto.build_point_id_request(point_id)

        response = await self._conn.call(
            self._conn.stub.GetPoint,
            request,
        )

        if response is None:
            return None

        return Point.from_proto(response)

    async def delete(self, *, point_id: str) -> None:
        """
        Delete a point by ID.
        """
        request = proto.build_point_id_request(point_id)

        await self._conn.call(
            self._conn.stub.DeletePoint,
            request,
        )

    async def search(
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
        if not isinstance(vector, DenseVector):
            raise TypeError(
                "vector must be a DenseVector. "
                "Use: DenseVector([1.0, 2.0, 3.0])"
            )

        request = proto.build_search_request(
            vector=vector,
            similarity=similarity,
            limit=limit,
        )

        response = await self._conn.call(
            self._conn.stub.SearchPoints,
            request,
        )

        return [pid.id.value for pid in response.result_point_ids]

    async def close(self) -> None:
        """
        Close the async gRPC connection.
        """
        await self._conn.close()

    async def __aenter__(self) -> "AsyncVortexDB":
        return self

    async def __aexit__(self, exc_type, exc, tb) -> None:
        await self.close()
