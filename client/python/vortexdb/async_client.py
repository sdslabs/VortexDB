from typing import List

from vortexdb import protoutils as proto
from vortexdb.async_connection import AsyncGRPCConnection
from vortexdb.config import VortexDBConfig
from vortexdb.models import DenseVector, Payload, Point, Similarity, SearchQuery


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
                "vector must be a DenseVector. Use: DenseVector([1.0, 2.0, 3.0])"
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

    async def batch_insert(
        self, *, items: list[tuple[DenseVector, Payload]]
    ) -> list[str]:
        """
        Insert multiple vectors.
        Returns: list of point_id (str)
        """
        request = proto.build_batch_insert_request(items=items)

        response = await self._conn.call(
            self._conn.stub.InsertVectorsBatch,
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

    async def search(
        self,
        *,
        vector: DenseVector | None = None,
        similarity: Similarity | None = None,
        limit: int | None = None,
        query: SearchQuery | None = None,
        ef: int | None = None,
    ) -> List[str]:
        """
        Search for nearest neighbors.
        Returns: List of point IDs
        """
        if query is not None:
            if not isinstance(query, SearchQuery):
                raise TypeError("query must be a SearchQuery")
            vector = query.vector
            similarity = query.similarity
            limit = query.limit
        else:
            if not isinstance(vector, DenseVector):
                raise TypeError(
                    "vector must be a DenseVector. Use: DenseVector([1.0, 2.0, 3.0])"
                )
            if not isinstance(similarity, Similarity):
                raise TypeError("similarity must be a Similarity enum")
            if not isinstance(limit, int):
                raise TypeError("limit must be an int")

        request = proto.build_search_request(
            vector=vector,
            similarity=similarity,
            limit=limit,
            ef=ef,
        )

        response = await self._conn.call(
            self._conn.stub.SearchPoints,
            request,
        )

        return [pid.id.value for pid in response.result_point_ids]

    async def batch_search(
        self,
        *,
        queries,
        similarity: Similarity | None = None,
        limit: int | None = None,
        ef: int | None = None,
    ) -> List[List[str]]:
        """
        Flexible batch search.

        Accepts:
        - List[SearchQuery]
        - List[(DenseVector, Similarity, int)]
        - List[(DenseVector, Similarity)] + global limit
        - List[(DenseVector, int)] + global similarity
        - List[DenseVector] + global similarity + limit
        """
        normalized = []

        for i, q in enumerate(queries):
            if (
                hasattr(q, "vector")
                and hasattr(q, "similarity")
                and hasattr(q, "limit")
            ):
                normalized.append((q.vector, q.similarity, q.limit))
                continue

            if isinstance(q, DenseVector):
                if similarity is None or limit is None:
                    raise ValueError(
                        f"queries[{i}] requires global similarity and limit"
                    )
                normalized.append((q, similarity, limit))
                continue

            if isinstance(q, (list, tuple)):
                if len(q) == 3:
                    normalized.append(q)
                    continue
                if len(q) == 2:
                    a, b = q

                    if isinstance(a, DenseVector) and isinstance(b, Similarity):
                        if limit is None:
                            raise ValueError(f"queries[{i}] missing global limit")
                        normalized.append((a, b, limit))
                        continue

                    if isinstance(a, DenseVector) and isinstance(b, int):
                        if similarity is None:
                            raise ValueError(f"queries[{i}] missing global similarity")
                        normalized.append((a, similarity, b))
                        continue

            raise TypeError(f"Invalid query format at index {i}")

        request = proto.build_batch_search_request(queries=normalized, ef=ef)
        response = await self._conn.call(self._conn.stub.SearchPointsBatch, request)
        return [
            [pid.id.value for pid in result.result_point_ids]
            for result in response.results
        ]

    async def close(self) -> None:
        """
        Close the async gRPC connection.
        """
        await self._conn.close()

    async def __aenter__(self) -> "AsyncVortexDB":
        return self

    async def __aexit__(self, exc_type, exc, tb) -> None:
        await self.close()
