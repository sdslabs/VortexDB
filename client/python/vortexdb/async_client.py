from vortexdb import protoutils as proto
from vortexdb.async_connection import AsyncGRPCConnection
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

    async def insert(
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

        response = await self._conn.call(
            self._conn.stub.InsertPoint,
            request,
        )

        return response.id.value

    async def batch_insert(self, *, points: list[PointInput]) -> list[str]:
        """
        Insert multiple points.
        Returns: list of point_id (str)
        """
        request = proto.build_batch_insert_request(points=points)

        response = await self._conn.call(
            self._conn.stub.InsertPointsBatch,
            request,
        )
        return [pid.id.value for pid in response.ids]

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

    async def search(self, *, query: SearchQuery) -> list[SearchHit]:
        """
        Search with a dense or sparse query.
        Returns: list of SearchHit (dense scores are distances, lower is better;
        sparse scores are dot products, higher is better)
        """
        request = proto.build_search_request(query)

        response = await self._conn.call(
            self._conn.stub.Search,
            request,
        )
        return proto.parse_search_response(response)

    async def batch_search(
        self, *, queries: list[SearchQuery]
    ) -> list[list[SearchHit]]:
        """
        Run multiple dense and/or sparse queries in one call.
        Returns: one list of SearchHit per query, in order
        """
        request = proto.build_batch_search_request(queries=queries)

        response = await self._conn.call(
            self._conn.stub.SearchBatch,
            request,
        )
        return [proto.parse_search_response(result) for result in response.results]

    async def close(self) -> None:
        """
        Close the async gRPC connection.
        """
        await self._conn.close()

    async def __aenter__(self) -> "AsyncVortexDB":
        return self

    async def __aexit__(self, exc_type, exc, tb) -> None:
        await self.close()
